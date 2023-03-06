window.SIDEBAR_ITEMS = {"attr":[["constructor","Attribute for functions run at program initialization (before main)."],["destructor","Attribute for functions run at program termination (after main)"],["dynamic","Declare statics that can be initialized with non const fonctions and safe mutable statics"]],"mod":[["lazy","Provides various implementation of lazily initialized types"],["phase","phases and bits to manipulate them;"],["raw_static","Provides types for statics that are meant to run code before main start or after it exit."]],"struct":[["AccessError","Lazy access error"],["Lazy","A type that initialize itself only once on the first access"],["LockedLazy","A mutable locked lazy that initialize its content on the first lock"],["Phase","The lifetime phase of an object, this indicate weither the object was initialized finalized (droped),…"],["UnSyncLazy","A version of [Lazy] whose reference can not be passed to other thread"],["UnSyncLockedLazy","A RefCell that initializes its content on the first access"]],"trait":[["Finaly","Trait that must be implemented by #[dynamic(finalize)] statics."],["Generator","Generates a value of type `T`"],["GeneratorTolerance",""],["LazyAccess","Helper trait to ease access static lazy associated functions"],["Phased","Trait for objects that know in which phase they are."],["Uninit","Trait that must be implemented by #[dynamic(prime)] mutable statics."]]};