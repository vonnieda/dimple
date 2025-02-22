use std::{error::Error, fmt::Display};

use rusqlite::Transaction;

/// - It's an offline first, reactive, synchronizable data store based on Sqlite.
pub trait DataStore {
    fn save<Model: DataStoreModel>(&self, model: Model) -> Result<Model, Box<dyn Error>>;
    fn get<Model: DataStoreModel>(&self, key: &str) -> Result<Option<Model>, Box<dyn Error>>;
    fn list<Model: DataStoreModel>(&self) -> Result<Vec<Model>, Box<dyn Error>>;
    fn query<Model: DataStoreModel>(&self, sql: &str) -> Result<Vec<Model>, Box<dyn Error>>;
    fn delete<Model: DataStoreModel>(&self, key: &str) -> Result<Option<Model>, Box<dyn Error>>;

    fn observe<Model: DataStoreModel>(&self, model: Model, callback: impl FnMut(DataStoreEvent) -> () + 'static);
    fn unobserve<Model: DataStoreModel>(&self, model: Model, callback: impl FnMut(DataStoreEvent) -> ());

    /// Sync:
    /// 1. Grab all remote changes.
    /// 2. Apply remote changes.
    /// 3. Collect changes since last sync.
    /// 4. Serialize, compress, and upload.
    /// 5. Mark last synced time.
    /// 
    /// So, given a sync path of s3://b2.backblaze.com/jvonnieda/dimple/
    /// And given the id of this library is 111111, and the other two devices
    /// participating are 222222 and 333333.
    /// 
    /// 1. List s3://b2.backblaze.com/jvonnieda/dimple/*.dimple_sync/*.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/111111.dimple_sync/aabbccddeeff000001.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/111111.dimple_sync/aabbccddeeff000002.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/111111.dimple_sync/aabbccddeeff000003.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/222222.dimple_sync/aabbccddeeff000001.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/222222.dimple_sync/aabbccddeeff000002.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/222222.dimple_sync/aabbccddeeff000003.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/222222.dimple_sync/aabbccddeeff000004.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/222222.dimple_sync/aabbccddeeff000005.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/333333.dimple_sync/aabbccddeeff000001.gz
    ///    s3://b2.backblaze.com/jvonnieda/dimple/333333.dimple_sync/aabbccddeeff000002.gz
    /// 2. Download any files that are newer (based on ulid in the filename) than what we
    ///    have for that actor.
    /// 3. Merge the objects in the files in order.
    ///    Making sure that what ends up in the changes table matches what was
    ///    in the files, so that we don't loop. I guess, for instance, that means
    ///    that when merging an object, if the results differ from the merged
    ///    one we need to mark that as changed, so that it gets re-shared. But
    ///    if the update results in no difference, we should set the changes
    ///    ulid to the greater of the two. I think.
    /// 4. Collect up all the changes since the last push and push them.
    /// 
    /// Essentially this is like a Git pull, merge, push workflow that uses
    /// CRDT rules for automatic conflict resolution.
    /// 
    /// So, if I get an object that is marked as older than what I already have
    /// what do I do?
    ///     I think it's just a LWW merge like I already have, with the newer
    ///     one getting priority for conflicts?
    /// Asides:
    /// - Will need a few tables to keep track of sync state I think. One for
    ///   last modified on each object, and one for keeping track of which sync
    ///   files have already been merged.

    // fn merge(&self, other: impl DataStore) -> Result<(), DataStoreError>;
    /// Returns the UUIDs? paths? name+uuid? of every model changed at or after the specified
    fn changes_since(&self, ulid: &str) -> Result<(), DataStoreError>;
}

// https://xuanwo.io/links/2025/02/aws_s3_sdk_breaks_its_compatible_services/
// https://github.com/awesomized/crc64fast-nvme


pub trait DataStoreModel {
    fn key(&self) -> Option<String>;
    fn set_key(&mut self, key: Option<String>);
    fn insert(&self, txn: &Transaction) -> Result<(), Box<dyn Error>>;
    fn update(&self, txn: &Transaction) -> Result<(), Box<dyn Error>>;
    fn type_name() -> String;
}

#[derive(Debug)]
pub enum DataStoreError {

}

impl Display for DataStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataStoreError")
    }
}

impl Error for DataStoreError {}

#[derive(Debug, Clone)]
pub enum DataStoreEvent {
    Created,
    Updated,
    Deleted,
}
