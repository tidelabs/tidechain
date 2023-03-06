window.SIDEBAR_ITEMS = {"constant":[["BABE_ENGINE_ID","The `ConsensusEngineId` of BABE."],["VRF_OUTPUT_LENGTH","Length of VRF output."]],"enum":[["BabeRequest","Requests to the BABE service."],["ConsensusLog","An consensus log item for BABE."],["Error","Errors encountered by the babe authorship task."],["NextConfigDescriptor","Information about the next epoch config, if changed. This is broadcast in the first block of the epoch, and applies using the same rules as `NextEpochDescriptor`."],["PreDigest","A BABE pre-runtime digest. This contains all data required to validate a block and for the BABE runtime module. Slots can be assigned to a primary (VRF based) and to a secondary (slot number based)."]],"fn":[["block_import","Produce a BABE block-import object to be used later on in the construction of an import-queue."],["configuration","Read configuration from the runtime state at current best block."],["find_pre_digest","Extract the BABE pre digest from the given header. Pre-runtime digests are mandatory, the function will return `Err` if none is found."],["import_queue","Start an import queue for the BABE consensus algorithm."],["revert","Reverts protocol aux data to at most the last finalized block. In particular, epoch-changes and block weights announced after the revert point are removed."],["start_babe","Start the babe worker."]],"mod":[["authorship","BABE authority selection and slot claiming."],["aux_schema","Schema for BABE epoch changes in the aux-db."]],"static":[["INTERMEDIATE_KEY","Intermediate key for Babe engine."]],"struct":[["BabeBlockImport","A block-import handler for BABE."],["BabeConfiguration","Configuration data used by the BABE consensus engine."],["BabeEpochConfiguration","Configuration data used by the BABE consensus engine that may change with epochs."],["BabeIntermediate","Intermediate value passed to block importer."],["BabeLink","State that must be shared between the import queue and the authoring logic."],["BabeParams","Parameters for BABE."],["BabeVerifier","A verifier for Babe blocks."],["BabeWorker","Worker for Babe which implements `Future<Output=()>`. This must be polled."],["BabeWorkerHandle","A handle to the BABE worker for issuing requests."],["Epoch","BABE epoch information"],["NextEpochDescriptor","Information about the next epoch. This is broadcast in the first block of the epoch."],["PrimaryPreDigest","Raw BABE primary slot assignment pre-digest."],["SecondaryPlainPreDigest","BABE secondary slot assignment pre-digest."],["SlotProportion","A unit type wrapper to express the proportion of a slot."]],"trait":[["BabeApi","API necessary for block authorship with BABE."],["CompatibleDigestItem","A digest item which is usable with BABE consensus."],["SyncOracle","An oracle for when major synchronization work is being undertaken."]],"type":[["AuthorityId","A Babe authority identifier. Necessarily equivalent to the schnorrkel public key used in the main Babe module. If that ever changes, then this must, too."],["AuthorityPair","A Babe authority keypair. Necessarily equivalent to the schnorrkel public key used in the main Babe module. If that ever changes, then this must, too."],["AuthoritySignature","A Babe authority signature."],["BabeAuthorityWeight","The weight of an authority."],["BabeBlockWeight","The cumulative weight of a BABE block, i.e. sum of block weights starting at this block until the genesis block."]]};