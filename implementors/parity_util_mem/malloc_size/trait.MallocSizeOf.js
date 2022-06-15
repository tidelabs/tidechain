(function() {var implementors = {};
implementors["kvdb_memorydb"] = [{"text":"impl <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"kvdb_memorydb/struct.InMemory.html\" title=\"struct kvdb_memorydb::InMemory\">InMemory</a>","synthetic":false,"types":["kvdb_memorydb::InMemory"]}];
implementors["kvdb_rocksdb"] = [{"text":"impl <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"kvdb_rocksdb/struct.Database.html\" title=\"struct kvdb_rocksdb::Database\">Database</a>","synthetic":false,"types":["kvdb_rocksdb::Database"]}];
implementors["memory_db"] = [{"text":"impl&lt;H, KF, T, M&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"memory_db/struct.MemoryDB.html\" title=\"struct memory_db::MemoryDB\">MemoryDB</a>&lt;H, KF, T, M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;H: <a class=\"trait\" href=\"hash_db/trait.Hasher.html\" title=\"trait hash_db::Hasher\">KeyHasher</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;H::<a class=\"associatedtype\" href=\"hash_db/trait.Hasher.html#associatedtype.Out\" title=\"type hash_db::Hasher::Out\">Out</a>: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;KF: <a class=\"trait\" href=\"memory_db/trait.KeyFunction.html\" title=\"trait memory_db::KeyFunction\">KeyFunction</a>&lt;H&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;KF::<a class=\"associatedtype\" href=\"memory_db/trait.KeyFunction.html#associatedtype.Key\" title=\"type memory_db::KeyFunction::Key\">Key</a>: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;M: <a class=\"trait\" href=\"memory_db/trait.MemTracker.html\" title=\"trait memory_db::MemTracker\">MemTracker</a>&lt;T&gt;,&nbsp;</span>","synthetic":false,"types":["memory_db::MemoryDB"]}];
implementors["sc_transaction_pool"] = [{"text":"impl&lt;B:&nbsp;<a class=\"trait\" href=\"sc_transaction_pool/trait.ChainApi.html\" title=\"trait sc_transaction_pool::ChainApi\">ChainApi</a>&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sc_transaction_pool/struct.Pool.html\" title=\"struct sc_transaction_pool::Pool\">Pool</a>&lt;B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;&lt;B as <a class=\"trait\" href=\"sc_transaction_pool/trait.ChainApi.html\" title=\"trait sc_transaction_pool::ChainApi\">ChainApi</a>&gt;::<a class=\"associatedtype\" href=\"sc_transaction_pool/trait.ChainApi.html#associatedtype.Block\" title=\"type sc_transaction_pool::ChainApi::Block\">Block</a> as <a class=\"trait\" href=\"sp_runtime/traits/trait.Block.html\" title=\"trait sp_runtime::traits::Block\">Block</a>&gt;::<a class=\"associatedtype\" href=\"sp_runtime/traits/trait.Block.html#associatedtype.Extrinsic\" title=\"type sp_runtime::traits::Block::Extrinsic\">Extrinsic</a>: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,&nbsp;</span>","synthetic":false,"types":["sc_transaction_pool::graph::pool::Pool"]},{"text":"impl&lt;Hash, Extrinsic&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sc_transaction_pool/struct.Transaction.html\" title=\"struct sc_transaction_pool::Transaction\">Transaction</a>&lt;Hash, Extrinsic&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Hash: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Extrinsic: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,&nbsp;</span>","synthetic":false,"types":["sc_transaction_pool::graph::base_pool::Transaction"]},{"text":"impl&lt;PoolApi, Block&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sc_transaction_pool/struct.BasicPool.html\" title=\"struct sc_transaction_pool::BasicPool\">BasicPool</a>&lt;PoolApi, Block&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;PoolApi: <a class=\"trait\" href=\"sc_transaction_pool/trait.ChainApi.html\" title=\"trait sc_transaction_pool::ChainApi\">ChainApi</a>&lt;Block = Block&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;Block: <a class=\"trait\" href=\"sp_runtime/traits/trait.Block.html\" title=\"trait sp_runtime::traits::Block\">BlockT</a>,&nbsp;</span>","synthetic":false,"types":["sc_transaction_pool::BasicPool"]}];
implementors["sp_runtime"] = [{"text":"impl&lt;Header, Extrinsic:&nbsp;<a class=\"trait\" href=\"sp_runtime/traits/trait.MaybeSerialize.html\" title=\"trait sp_runtime::traits::MaybeSerialize\">MaybeSerialize</a>&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sp_runtime/generic/struct.Block.html\" title=\"struct sp_runtime::generic::Block\">Block</a>&lt;Header, Extrinsic&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Header: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Extrinsic: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,&nbsp;</span>","synthetic":false,"types":["sp_runtime::generic::block::Block"]},{"text":"impl <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sp_runtime/generic/struct.Digest.html\" title=\"struct sp_runtime::generic::Digest\">Digest</a>","synthetic":false,"types":["sp_runtime::generic::digest::Digest"]},{"text":"impl <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"enum\" href=\"sp_runtime/generic/enum.DigestItem.html\" title=\"enum sp_runtime::generic::DigestItem\">DigestItem</a>","synthetic":false,"types":["sp_runtime::generic::digest::DigestItem"]},{"text":"impl&lt;Number, Hash&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sp_runtime/generic/struct.Header.html\" title=\"struct sp_runtime::generic::Header\">Header</a>&lt;Number, Hash&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Number: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"primitive_types/struct.U256.html\" title=\"struct primitive_types::U256\">U256</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.61.0/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"struct\" href=\"primitive_types/struct.U256.html\" title=\"struct primitive_types::U256\">U256</a>&gt; + <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Hash: <a class=\"trait\" href=\"sp_runtime/traits/trait.Hash.html\" title=\"trait sp_runtime::traits::Hash\">HashT</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Hash::<a class=\"associatedtype\" href=\"sp_runtime/traits/trait.Hash.html#associatedtype.Output\" title=\"type sp_runtime::traits::Hash::Output\">Output</a>: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,&nbsp;</span>","synthetic":false,"types":["sp_runtime::generic::header::Header"]},{"text":"impl&lt;Address, Call, Signature, Extra&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sp_runtime/generic/struct.UncheckedExtrinsic.html\" title=\"struct sp_runtime::generic::UncheckedExtrinsic\">UncheckedExtrinsic</a>&lt;Address, Call, Signature, Extra&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Extra: <a class=\"trait\" href=\"sp_runtime/traits/trait.SignedExtension.html\" title=\"trait sp_runtime::traits::SignedExtension\">SignedExtension</a>,&nbsp;</span>","synthetic":false,"types":["sp_runtime::generic::unchecked_extrinsic::UncheckedExtrinsic"]},{"text":"impl&lt;Xt&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sp_runtime/testing/struct.ExtrinsicWrapper.html\" title=\"struct sp_runtime::testing::ExtrinsicWrapper\">ExtrinsicWrapper</a>&lt;Xt&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Xt: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,&nbsp;</span>","synthetic":false,"types":["sp_runtime::testing::ExtrinsicWrapper"]},{"text":"impl&lt;Xt&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sp_runtime/testing/struct.Block.html\" title=\"struct sp_runtime::testing::Block\">Block</a>&lt;Xt&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Xt: <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a>,&nbsp;</span>","synthetic":false,"types":["sp_runtime::testing::Block"]},{"text":"impl&lt;Call, Extra&gt; <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sp_runtime/testing/struct.TestXt.html\" title=\"struct sp_runtime::testing::TestXt\">TestXt</a>&lt;Call, Extra&gt;","synthetic":false,"types":["sp_runtime::testing::TestXt"]},{"text":"impl <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"enum\" href=\"sp_runtime/transaction_validity/enum.TransactionSource.html\" title=\"enum sp_runtime::transaction_validity::TransactionSource\">TransactionSource</a>","synthetic":false,"types":["sp_runtime::transaction_validity::TransactionSource"]},{"text":"impl <a class=\"trait\" href=\"parity_util_mem/malloc_size/trait.MallocSizeOf.html\" title=\"trait parity_util_mem::malloc_size::MallocSizeOf\">MallocSizeOf</a> for <a class=\"struct\" href=\"sp_runtime/struct.OpaqueExtrinsic.html\" title=\"struct sp_runtime::OpaqueExtrinsic\">OpaqueExtrinsic</a>","synthetic":false,"types":["sp_runtime::OpaqueExtrinsic"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()