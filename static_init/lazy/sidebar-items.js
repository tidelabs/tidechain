window.SIDEBAR_ITEMS = {"mod":[["global_primed_locked_lazy_droped",""],["lesser_locked_lazy",""],["lesser_locked_lazy_droped",""],["lesser_locked_lazy_finalize",""],["locked_lazy",""],["locked_lazy_droped",""],["locked_lazy_finalize",""],["primed_lesser_locked_lazy",""],["primed_locked_lazy",""],["primed_locked_lazy_droped",""],["unsync_locked_lazy",""]],"struct":[["Lazy","A type that initialize itself only once on the first access"],["LazyFinalize","The actual type of statics attributed with #[dynamic(lazy,finalize)] The method from_generator is unsafe as the object must be a non mutable static."],["LesserLazy","The actual type of statics attributed with #[dynamic]. The method from_generator is unsafe because this kind of static can only safely be used through this attribute macros."],["LesserLazyFinalize","The actual type of statics attributed with #[dynamic(finalize)]. The method from_generator is unsafe because this kind of static can only safely be used through this attribute macros."],["LesserLockedLazy","The actual type of mutable statics attributed with #[dynamic] The method from_generator is unsafe because this kind of static can only safely be used through this attribute macros."],["LesserLockedLazyDroped","The actual type of mutable statics attributed with #[dynamic(drop)] The method (new)[Self::from_generator] is unsafe because this kind of static can only safely be used through this attribute macros."],["LesserLockedLazyFinalize","The actual type of mutable statics attributed with #[dynamic(finalize)] The method from_generator is unsafe because this kind of static can only safely be used through this attribute macros."],["LockedLazy","A mutable locked lazy that initialize its content on the first lock"],["LockedLazyDroped","The actual type of statics attributed with #[dynamic(lazy,finalize)]"],["LockedLazyFinalize","The actual type of mutable statics attributed with #[dynamic(lazy,finalize)]"],["PrimedLesserLockedLazy","The actual type of mutable statics attributed with #[dynamic(primed)]"],["PrimedLesserLockedLazyDroped","The actual type of mutable statics attributed with #[dynamic(primed,drop)]"],["PrimedLockedLazy","The actual type of mutable statics attributed with #[dynamic(primed)]"],["PrimedLockedLazyDroped","The actual type of mutable statics attributed with #[dynamic(primed,drop)]"],["UnSyncLazy","A version of [Lazy] whose reference can not be passed to other thread"],["UnSyncLockedLazy","A RefCell that initializes its content on the first access"]],"trait":[["LazyAccess","Helper trait to ease access static lazy associated functions"]]};