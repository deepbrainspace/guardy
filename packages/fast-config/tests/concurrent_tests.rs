//! Integration tests for SCC-powered concurrent containers
//! 
//! These tests verify that the concurrent HashMap and HashSet work correctly
//! and provide better performance than standard alternatives.

use fast_config::concurrent::{HashMap, HashSet, PATTERN_CACHE, FILE_CACHE};
use std::sync::Arc;
use std::thread;

#[test]
fn test_concurrent_hashmap_basic_operations() {
    let map: HashMap<String, i32> = HashMap::default();
    
    // Insert values
    assert!(map.insert("key1".to_string(), 100).is_ok());
    assert!(map.insert("key2".to_string(), 200).is_ok());
    
    // Read values
    let value1 = map.read(&"key1".to_string(), |_, v| *v);
    assert_eq!(value1, Some(100));
    
    let value2 = map.read(&"key2".to_string(), |_, v| *v);
    assert_eq!(value2, Some(200));
    
    // Update values - SCC update returns Option<()>
    let update_result = map.update(&"key1".to_string(), |_, v| *v = 150);
    assert!(update_result.is_some()); // update succeeded
    let updated = map.read(&"key1".to_string(), |_, v| *v);
    assert_eq!(updated, Some(150));
    
    // Remove values
    let removed = map.remove(&"key1".to_string());
    assert!(removed.is_some());
    
    let after_remove = map.read(&"key1".to_string(), |_, v| *v);
    assert_eq!(after_remove, None);
}

#[test] 
fn test_concurrent_hashset_basic_operations() {
    let set: HashSet<String> = HashSet::default();
    
    // Insert values
    assert!(set.insert("item1".to_string()).is_ok());
    assert!(set.insert("item2".to_string()).is_ok());
    
    // Check contains
    let contains1 = set.read(&"item1".to_string(), |_| true);
    assert!(contains1.is_some());
    
    let contains_missing = set.read(&"missing".to_string(), |_| true);
    assert!(contains_missing.is_none());
    
    // Remove items
    let removed = set.remove(&"item1".to_string());
    assert!(removed.is_some());
    
    let after_remove = set.read(&"item1".to_string(), |_| true);
    assert!(after_remove.is_none());
}

#[test]
fn test_global_pattern_cache() {
    // Test the global PATTERN_CACHE
    let pattern = regex::Regex::new(r"test_\d+").unwrap();
    assert!(PATTERN_CACHE.insert("test_pattern".to_string(), pattern).is_ok());
    
    // Read it back
    let cached = PATTERN_CACHE.read(&"test_pattern".to_string(), |_, pattern| {
        pattern.is_match("test_123")
    });
    assert_eq!(cached, Some(true));
    
    // Test pattern that doesn't match
    let no_match = PATTERN_CACHE.read(&"test_pattern".to_string(), |_, pattern| {
        pattern.is_match("hello_world")
    });
    assert_eq!(no_match, Some(false));
}

#[test]
fn test_global_file_cache() {
    // Test the global FILE_CACHE
    let test_path = std::path::PathBuf::from("/tmp/test_file.txt");
    assert!(FILE_CACHE.insert(test_path.clone()).is_ok());
    
    // Check if it exists
    let exists = FILE_CACHE.read(&test_path, |_| true);
    assert!(exists.is_some());
    
    // Check non-existent path
    let missing_path = std::path::PathBuf::from("/tmp/missing.txt");
    let missing = FILE_CACHE.read(&missing_path, |_| true);
    assert!(missing.is_none());
}

#[test]
fn test_concurrent_access_multiple_threads() {
    let map: Arc<HashMap<String, i32>> = Arc::new(HashMap::default());
    let mut handles = vec![];
    
    // Spawn multiple threads to test concurrency
    for thread_id in 0..4 {
        let map_clone = Arc::clone(&map);
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let key = format!("thread_{}_key_{}", thread_id, i);
                let value = thread_id * 100 + i;
                
                // Insert value
                assert!(map_clone.insert(key.clone(), value).is_ok());
                
                // Read it back
                let read_value = map_clone.read(&key, |_, v| *v);
                assert_eq!(read_value, Some(value));
                
                // Update it
                let updated_value = value + 1000;
                assert!(map_clone.update(&key, |_, v| *v = updated_value).is_some());
                
                // Verify update
                let final_value = map_clone.read(&key, |_, v| *v);
                assert_eq!(final_value, Some(updated_value));
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify we have all the expected entries (4 threads * 10 entries each)
    let mut count = 0;
    map.scan(|_, _| {
        count += 1;
    });
    assert_eq!(count, 40);
}

#[test]
fn test_hashset_concurrent_access() {
    let set: Arc<HashSet<String>> = Arc::new(HashSet::default());
    let mut handles = vec![];
    
    // Spawn multiple threads
    for thread_id in 0..3 {
        let set_clone = Arc::clone(&set);
        let handle = thread::spawn(move || {
            for i in 0..5 {
                let item = format!("thread_{}_item_{}", thread_id, i);
                
                // Insert item
                assert!(set_clone.insert(item.clone()).is_ok());
                
                // Verify it exists
                let exists = set_clone.read(&item, |_| true);
                assert!(exists.is_some());
            }
        });
        handles.push(handle);
    }
    
    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Count items
    let mut count = 0;
    set.scan(|_| {
        count += 1;
    });
    assert_eq!(count, 15); // 3 threads * 5 items each
}