extern crate lazy_static;
use lazy_static::lazy_static;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::any::Any;
use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

/**
 * How this works is:
 * 
 * Data entry is the raw base recieving a raw C generic pointer. Idk of any type capable of this in rust? but idk who knows.
 * 
 * Then, the 'inheritors' in TypedDataEntry and TypedObjectDataEntry inherit the base, and implement it so that they cast their types to their 
 * generic type before calling their callback. This way, as long as we set the data up correctly in the hash maps, we can map Type <--> Type from
 * the subscribers and publishers.
 */

trait DataEntry {
  fn receive(& mut self, thing: * const std::ffi::c_void);
}

struct TypedDataEntry<T> {
  callback: fn(&T),
}

impl<T> DataEntry for TypedDataEntry<T> {
  fn receive(& mut self, thing: * const std::ffi::c_void) {
    unsafe {
      let casted = &*(thing as * const T);
      (self.callback)(casted);
    }
  }
}

struct TypedObjectDataEntry<T, O: Send> {

  object: AtomicPtr<O>,
  callback: fn(& mut O, &T),
}

impl<T, O: Send> DataEntry for TypedObjectDataEntry<T, O> {
  fn receive(& mut self, thing: * const std::ffi::c_void) {
    unsafe {
      let casted = &*(thing as * const T);
      let object = & mut *(*(self.object.get_mut()));
      (self.callback)(object, casted);
    }
  }
}

type TypeCallbacks = HashMap<std::any::TypeId, HashMap<u64, Box<dyn DataEntry + Send>>>;

struct DataBusMappings {
  pub callbacks: HashMap<String, TypeCallbacks>,
  pub counter: u64,
}

impl Default for DataBusMappings {
  fn default() -> Self {
    return DataBusMappings {
      callbacks: Default::default(),
      counter: 0,
    }
  }
}

lazy_static! {
  static ref BUS_DATA: std::sync::Mutex<DataBusMappings> = Default::default();
}

pub struct DataBus {
  local_mappings: HashMap<String, HashMap<std::any::TypeId, u64>>,
}

impl Default for DataBus {
  fn default() -> Self {
      return DataBus { local_mappings: Default::default() };
  }
}

impl Drop for DataBus {
  fn drop(&mut self) {
    let mut data = BUS_DATA.lock()
    .expect("Unable to recieve data bus mutex!");

    for entry in & mut self.local_mappings {
      let mut global_key_lookup = & mut data.callbacks.get_mut(entry.0)
      .expect("Failed global lookup!");

      // Now we have the same key : key. 
      // We need to go over the stored 'types' we have stored up now and remove them all.

      for type_callback in entry.1 {
        let type_lookup = global_key_lookup.get_mut(type_callback.0)
        .expect("Unable to find data bus type mapping!");
        
        type_lookup.remove_entry(type_callback.1);
      }
    }
  }
}

impl DataBus {
  pub fn new() {
    return ();
  }

  pub fn send<T: Any>(&self, key: &str, in_data: &T) {
    let mut data = BUS_DATA.lock()
    .expect("Unable to recieve data bus mutex!");

    if data.callbacks.contains_key(&key.to_string()) {
      let type_entries = data.callbacks.get_mut(key).unwrap();

      let typeid = std::any::TypeId::of::<T>();
      if type_entries.contains_key(&typeid) {
        let type_cbs = type_entries.get_mut(&typeid).unwrap();
        
        for cb in type_cbs {
          cb.1.receive(in_data as * const T as * const c_void);
        }
      }
    }
  }

  pub fn add_subscriber<T: Any>(& mut self, key: &str, callback: fn(&T)) {
    let mut data = BUS_DATA.lock()
    .expect("Unable to recieve data bus mutex!");

    let id = data.counter;

    if !data.callbacks.contains_key(key) {
      data.callbacks.insert(key.to_string(), Default::default());
    } 

    if !self.local_mappings.contains_key(key) {
      self.local_mappings.insert(key.to_string(), Default::default());
    }

    let type_entries = data.callbacks.get_mut(key).unwrap();
    let local_type_entries = self.local_mappings.get_mut(key).unwrap();

    let typeid = std::any::TypeId::of::<T>();
    if !type_entries.contains_key(&typeid) {
      type_entries.insert(typeid, Default::default());
    }

    // We can only have one entry per type per object
    if !local_type_entries.contains_key(&typeid) {
      let type_entry = type_entries.get_mut(&typeid).unwrap();
      let entry = Box::new(TypedDataEntry{callback: callback});
      local_type_entries.insert(typeid, id);
      type_entry.insert(id, entry);
      data.counter += 1;
    }
  }

  pub fn add_object_subscriber<T: Any, O: Any + Send>(& mut self, key: &str, object: * mut O, callback: fn(& mut O, &T)) {
    let mut data = BUS_DATA.lock()
    .expect("Unable to recieve data bus mutex!");

    let id = data.counter;

    if !data.callbacks.contains_key(key) {
      data.callbacks.insert(key.to_string(), Default::default());
    } 

    if !self.local_mappings.contains_key(key) {
      self.local_mappings.insert(key.to_string(), Default::default());
    }

    let type_entries = data.callbacks.get_mut(key).unwrap();
    let local_type_entries = self.local_mappings.get_mut(key).unwrap();

    let typeid = std::any::TypeId::of::<T>();
    if !type_entries.contains_key(&typeid) {
      type_entries.insert(typeid, Default::default());
    }

    // We can only have one entry per type per object
    if !local_type_entries.contains_key(&typeid) {
      let type_entry = type_entries.get_mut(&typeid).unwrap();
      let entry = Box::new(TypedObjectDataEntry{callback: callback, object: AtomicPtr::new(object)});
      local_type_entries.insert(typeid, id);
      type_entry.insert(id, entry);
      data.counter += 1;
    }
  }
}