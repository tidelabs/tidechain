window.SIDEBAR_ITEMS = {"constant":[["MAX_EXTRINSIC_DEPTH","Maximum nesting level for extrinsics."],["RUNTIME_API_INFO_SIZE","The number of bytes required to encode a [`RuntimeApiInfo`]."]],"enum":[["ApiError","An error describing which API call failed."],["StateVersion","Different possible state version."]],"fn":[["deserialize_runtime_api_info","Deserialize the runtime API info serialized by [`serialize_runtime_api_info`]."],["init_runtime_logger","Init the `RuntimeLogger`."],["serialize_runtime_api_info","Crude and simple way to serialize the `RuntimeApiInfo` into a bunch of bytes."]],"macro":[["decl_runtime_apis","Declares given traits as runtime apis."],["impl_runtime_apis","Tags given trait implementations as runtime apis."],["mock_impl_runtime_apis","Mocks given trait implementations as runtime apis."]],"struct":[["ApiRef","Auxiliary wrapper that holds an api instance and binds it to the given lifetime."],["CallApiAtParams","Parameters for [`CallApiAt::call_api_at`]."]],"trait":[["ApiExt","Extends the runtime api implementation with some common functionality."],["CallApiAt","Something that can call into the an api at a given block."],["ConstructRuntimeApi","Something that can be constructed to a runtime api."],["Core","The `Core` runtime api that every Substrate runtime needs to implement."],["Metadata","The `Metadata` api trait that returns metadata for the runtime."],["ProvideRuntimeApi","Something that provides a runtime api."],["RuntimeApiInfo","Something that provides information about a runtime api."]],"type":[["ProofRecorder","A type that records all accessed trie nodes and generates a proof out of it."],["StateBackendFor","Extract the state backend type for a type that implements `ProvideRuntimeApi`."],["StorageChanges",""],["StorageTransactionCache","A type that is used as cache for the storage transactions."],["TransactionFor","Extract the state backend transaction type for a type that implements `ProvideRuntimeApi`."]]};