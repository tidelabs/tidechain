window.SIDEBAR_ITEMS = {"constant":[["DEFAULT_COLUMN_FAMILY_NAME","The name of the default column family."]],"enum":[["BlockBasedIndexType","Used by BlockBasedOptions::set_index_type."],["BottommostLevelCompaction",""],["DBCompactionStyle",""],["DBCompressionType",""],["DBRecoveryMode",""],["DataBlockIndexType","Used by BlockBasedOptions::set_data_block_index_type."],["Direction",""],["IteratorMode",""],["LogLevel",""],["MemtableFactory","Defines the underlying memtable implementation. See official wiki for more information."],["UniversalCompactionStopStyle",""]],"mod":[["backup",""],["checkpoint","Implementation of bindings to RocksDB Checkpoint1 API"],["compaction_filter",""],["compaction_filter_factory",""],["merge_operator","rustic merge operator"],["perf",""],["properties","Properties"]],"struct":[["BlockBasedOptions","For configuring block-based file storage."],["BoundColumnFamily","A specialized opaque type used to represent a column family by the [`MultiThreaded`] mode. Clone (and Copy) is derived to behave like `&ColumnFamily` (this is used for single-threaded mode). `Clone`/`Copy` is safe because this lifetime is bound to DB like iterators/snapshots. On top of it, this is as cheap and small as `&ColumnFamily` because this only has a single pointer-wide field."],["Cache",""],["ColumnFamily","An opaque type used to represent a column family. Returned from some functions, and used in others"],["ColumnFamilyDescriptor","A descriptor for a RocksDB column family."],["CompactOptions",""],["CuckooTableOptions","Configuration of cuckoo-based storage."],["DBIteratorWithThreadMode","An iterator over a database or column family, with specifiable ranges and direction."],["DBPath","Represents a path where sst files can be put into"],["DBPinnableSlice","Wrapper around RocksDB PinnableSlice struct."],["DBRawIteratorWithThreadMode","An iterator over a database or column family, with specifiable ranges and direction."],["DBWALIterator","Iterates the batches of writes since a given sequence number."],["DBWithThreadMode","A RocksDB database."],["Env","An Env is an interface used by the rocksdb implementation to access operating system functionality like the filesystem etc.  Callers may wish to provide a custom Env object when opening a database to get fine gain control; e.g., to rate limit file system operations."],["Error","A simple wrapper round a string, used for errors reported from ffi calls."],["FifoCompactOptions",""],["FlushOptions","Optionally wait for the memtable flush to be performed."],["IngestExternalFileOptions","For configuring external files ingestion."],["LiveFile","The metadata that describes a SST file"],["MultiThreaded","Actual marker type for the marker trait `ThreadMode`, which holds a collection of column families wrapped in a RwLock to be mutated concurrently. The other mode is [`SingleThreaded`]."],["Options","Database-wide options around performance and behavior."],["PlainTableFactoryOptions","Used with DBOptions::set_plain_table_factory. See official wiki for more information."],["ReadOptions",""],["SingleThreaded","Actual marker type for the marker trait `ThreadMode`, which holds a collection of column families without synchronization primitive, providing no overhead for the single-threaded column family alternations. The other mode is [`MultiThreaded`]."],["SliceTransform","A `SliceTransform` is a generic pluggable way of transforming one string to another. Its primary use-case is in configuring rocksdb to store prefix blooms by setting prefix_extractor in ColumnFamilyOptions."],["SnapshotWithThreadMode","A consistent view of the database at the point of creation."],["SstFileWriter","SstFileWriter is used to create sst files that can be added to database later All keys in files generated by SstFileWriter will have sequence number = 0."],["UniversalCompactOptions",""],["WriteBatch","An atomic batch of write operations."],["WriteOptions","Optionally disable WAL or sync for this write."]],"trait":[["AsColumnFamilyRef","Utility trait to accept both supported references to `ColumnFamily` (`&ColumnFamily` and `BoundColumnFamily`)"],["DBAccess","Minimal set of DB-related methods, intended to be  generic over `DBWithThreadMode<T>`. Mainly used internally"],["ThreadMode","Marker trait to specify single or multi threaded column family alternations for [`DBWithThreadMode<T>`]"],["WriteBatchIterator","Receives the puts and deletes of a write batch."]],"type":[["ColumnFamilyRef","Handy type alias to hide actual type difference to reference [`ColumnFamily`] depending on the `multi-threaded-cf` crate feature."],["DB","A type alias to DB instance type with the single-threaded column family creations/deletions"],["DBIterator","A type alias to keep compatibility. See [`DBIteratorWithThreadMode`] for details"],["DBRawIterator","A type alias to keep compatibility. See [`DBRawIteratorWithThreadMode`] for details"],["Snapshot","A type alias to keep compatibility. See [`SnapshotWithThreadMode`] for details"]]};