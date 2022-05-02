initSidebarItems({"struct":[["Arc","A thread-safe reference-counting pointer. ‘Arc’ stands for ‘Atomically Reference Counted’."],["Barrier","A counter to synchronize multiple tasks at the same time."],["BarrierWaitResult","Returned by [`Barrier::wait()`] when all tasks have called it."],["Condvar","A Condition Variable"],["Mutex","An async mutex."],["MutexGuard","A guard that releases the mutex when dropped."],["MutexGuardArc","An owned guard that releases the mutex when dropped."],["RwLock","An async reader-writer lock."],["RwLockReadGuard","A guard that releases the read lock when dropped."],["RwLockUpgradableReadGuard","A guard that releases the upgradable read lock when dropped."],["RwLockWriteGuard","A guard that releases the write lock when dropped."],["Weak","`Weak` is a version of [`Arc`] that holds a non-owning reference to the managed allocation. The allocation is accessed by calling `upgrade` on the `Weak` pointer, which returns an [Option]<[Arc]<T>>."]]});