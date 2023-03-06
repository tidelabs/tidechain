window.SIDEBAR_ITEMS = {"attr":[["benchmark","An attribute macro used to declare a benchmark within a benchmarking module. Must be attached to a function definition containing an `#[extrinsic_call]` or `#[block]` attribute."],["benchmarks","An attribute macro that can be attached to a (non-empty) module declaration. Doing so will designate that module as a benchmarking module."],["block","An attribute macro used to specify that a block should be the measured portion of the enclosing benchmark function, This attribute is also used as a boundary designating where the benchmark setup code ends, and the benchmark verification code begins."],["call_index","Each dispatchable may also be annotated with the `#[pallet::call_index($idx)]` attribute, which explicitly defines the codec index for the dispatchable function in the `Call` enum."],["compact","Compact encoding for arguments can be achieved via `#[pallet::compact]`. The function must return a `DispatchResultWithPostInfo` or `DispatchResult`."],["config","The mandatory attribute `#[pallet::config]` defines the configurable options for the pallet."],["constant","The `#[pallet::constant]` attribute can be used to add an associated type trait bounded by `Get` from `pallet::config` into metadata, e.g.:"],["disable_frame_system_supertrait_check","To bypass the `frame_system::Config` supertrait check, use the attribute `pallet::disable_frame_system_supertrait_check`, e.g.:"],["error","The `#[pallet::error]` attribute allows you to define an error enum that will be returned from the dispatchable when an error occurs. The information for this error type is then stored in metadata."],["event","The `#[pallet::event]` attribute allows you to define pallet events. Pallet events are stored under the `system` / `events` key when the block is applied (and then replaced when the next block writes it’s events)."],["extra_constants","Allows you to define some extra constants to be added into constant metadata."],["extrinsic_call","An attribute macro used to specify the extrinsic call inside a benchmark function, and also used as a boundary designating where the benchmark setup code ends, and the benchmark verification code begins."],["generate_deposit","The attribute `#[pallet::generate_deposit($visibility fn deposit_event)]` generates a helper function on `Pallet` that handles deposit events."],["generate_storage_info","To generate the full storage info (used for PoV calculation) use the attribute `#[pallet::generate_storage_info]`, e.g.:"],["generate_store","To generate a `Store` trait associating all storages, annotate your `Pallet` struct with the attribute `#[pallet::generate_store($vis trait Store)]`, e.g.:"],["genesis_build","The `#[pallet::genesis_build]` attribute allows you to define how `genesis_configuration` is built. This takes as input the `GenesisConfig` type (as `self`) and constructs the pallet’s initial state."],["genesis_config","The `#[pallet::genesis_config]` attribute allows you to define the genesis configuration for the pallet."],["getter","The optional attribute `#[pallet::getter(fn $my_getter_fn_name)]` allows you to define a getter function on `Pallet`."],["hooks","The `#[pallet::hooks]` attribute allows you to specify a `Hooks` implementation for `Pallet` that specifies pallet-specific logic."],["inherent","The `#[pallet::inherent]` attribute allows the pallet to provide some inherent. An inherent is some piece of data that is inserted by a block authoring node at block creation time and can either be accepted or rejected by validators based on whether the data falls within an acceptable range."],["instance_benchmarks","An attribute macro that can be attached to a (non-empty) module declaration. Doing so will designate that module as an instance benchmarking module."],["origin","The `#[pallet::origin]` attribute allows you to define some origin for the pallet."],["pallet","The pallet struct placeholder `#[pallet::pallet]` is mandatory and allows you to specify pallet information."],["require_transactional",""],["storage","The `#[pallet::storage]` attribute lets you define some abstract storage inside of runtime storage and also set its metadata. This attribute can be used multiple times."],["storage_alias",""],["storage_prefix","The optional attribute `#[pallet::storage_prefix = \"SomeName\"]` allows you to define the storage prefix to use. This is helpful if you wish to rename the storage field but don’t want to perform a migration."],["storage_version","Because the `pallet::pallet` macro implements `GetStorageVersion`, the current storage version needs to be communicated to the macro. This can be done by using the `pallet::storage_version` attribute:"],["transactional","Execute the annotated function in a new storage transaction."],["type_value","The `#[pallet::type_value]` attribute lets you define a struct implementing the `Get` trait to ease the use of storage types. This attribute is meant to be used alongside `#[pallet::storage]` to define a storage’s default value. This attribute can be used multiple times."],["unbounded","The optional attribute `#[pallet::unbounded]` declares the storage as unbounded. When implementating the storage info (when `#[pallet::generate_storage_info]` is specified on the pallet struct placeholder), the size of the storage will be declared as unbounded. This can be useful for storage which can never go into PoV (Proof of Validity)."],["validate_unsigned","The `#[pallet::validate_unsigned]` attribute allows the pallet to validate some unsigned transaction:"],["weight","Each dispatchable needs to define a weight with `#[pallet::weight($expr)]` attribute, the first argument must be `origin: OriginFor<T>`."],["whitelist_storage","The optional attribute `#[pallet::whitelist_storage]` will declare the storage as whitelisted from benchmarking. Doing so will exclude reads of that value’s storage key from counting towards weight calculations during benchmarking."]],"derive":[["CloneNoBound","Derive [`Clone`] but do not bound any generic. Docs are at `frame_support::CloneNoBound`."],["DebugNoBound","Derive [`Debug`] but do not bound any generics. Docs are at `frame_support::DebugNoBound`."],["DefaultNoBound","derive `Default` but do no bound any generic. Docs are at `frame_support::DefaultNoBound`."],["EqNoBound","derive Eq but do no bound any generic. Docs are at `frame_support::EqNoBound`."],["PalletError",""],["PartialEqNoBound","Derive [`PartialEq`] but do not bound any generic. Docs are at `frame_support::PartialEqNoBound`."],["RuntimeDebugNoBound","Derive [`Debug`], if `std` is enabled it uses `frame_support::DebugNoBound`, if `std` is not enabled it just returns `\"<stripped>\"`. This behaviour is useful to prevent bloating the runtime WASM blob from unneeded code."]],"macro":[["__create_tt_macro","Internal macro used by `frame_support` to create tt-call-compliant macros"],["__generate_dummy_part_checker","Internal macro use by frame_support to generate dummy part checker for old pallet declaration"],["construct_runtime","Construct a runtime, with the given name and the given pallets."],["crate_to_crate_version",""],["decl_storage","Declares strongly-typed wrappers around codec-compatible types in storage."],["impl_key_prefix_for_tuples","This macro is meant to be used by frame-support only. It implements the trait `HasKeyPrefix` and `HasReversibleKeyPrefix` for tuple of `Key`."],["match_and_insert","Macro that inserts some tokens after the first match of some pattern."]]};