(function() {var implementors = {
"curve25519_dalek":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;T&gt; for <a class=\"struct\" href=\"curve25519_dalek/scalar/struct.Scalar.html\" title=\"struct curve25519_dalek::scalar::Scalar\">Scalar</a><span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;<a class=\"struct\" href=\"curve25519_dalek/scalar/struct.Scalar.html\" title=\"struct curve25519_dalek::scalar::Scalar\">Scalar</a>&gt;,</span>"]],
"js_sys":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;<a class=\"struct\" href=\"js_sys/struct.BigInt.html\" title=\"struct js_sys::BigInt\">BigInt</a>&gt; for <a class=\"struct\" href=\"js_sys/struct.BigInt.html\" title=\"struct js_sys::BigInt\">BigInt</a>"],["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;&amp;'a <a class=\"struct\" href=\"js_sys/struct.BigInt.html\" title=\"struct js_sys::BigInt\">BigInt</a>&gt; for <a class=\"struct\" href=\"js_sys/struct.BigInt.html\" title=\"struct js_sys::BigInt\">BigInt</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;<a class=\"struct\" href=\"js_sys/struct.Number.html\" title=\"struct js_sys::Number\">Number</a>&gt; for <a class=\"struct\" href=\"js_sys/struct.Number.html\" title=\"struct js_sys::Number\">Number</a>"],["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;&amp;'a <a class=\"struct\" href=\"js_sys/struct.Number.html\" title=\"struct js_sys::Number\">Number</a>&gt; for <a class=\"struct\" href=\"js_sys/struct.Number.html\" title=\"struct js_sys::Number\">Number</a>"]],
"nalgebra":[["impl&lt;T, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;<a class=\"struct\" href=\"nalgebra/base/struct.Matrix.html\" title=\"struct nalgebra::base::Matrix\">Matrix</a>&lt;T, D, D, &lt;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a> as <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;T, D, D&gt;&gt;::<a class=\"associatedtype\" href=\"nalgebra/base/allocator/trait.Allocator.html#associatedtype.Buffer\" title=\"type nalgebra::base::allocator::Allocator::Buffer\">Buffer</a>&gt;&gt; for <a class=\"type\" href=\"nalgebra/base/type.OMatrix.html\" title=\"type nalgebra::base::OMatrix\">OMatrix</a>&lt;T, D, D&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a> + <a class=\"trait\" href=\"num_traits/identities/trait.Zero.html\" title=\"trait num_traits::identities::Zero\">Zero</a> + <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedMul.html\" title=\"trait nalgebra::ClosedMul\">ClosedMul</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedAdd.html\" title=\"trait nalgebra::ClosedAdd\">ClosedAdd</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;T, D, D&gt;,</span>"],["impl&lt;'a, T, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;&amp;'a <a class=\"struct\" href=\"nalgebra/base/struct.Matrix.html\" title=\"struct nalgebra::base::Matrix\">Matrix</a>&lt;T, D, D, &lt;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a> as <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;T, D, D&gt;&gt;::<a class=\"associatedtype\" href=\"nalgebra/base/allocator/trait.Allocator.html#associatedtype.Buffer\" title=\"type nalgebra::base::allocator::Allocator::Buffer\">Buffer</a>&gt;&gt; for <a class=\"type\" href=\"nalgebra/base/type.OMatrix.html\" title=\"type nalgebra::base::OMatrix\">OMatrix</a>&lt;T, D, D&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a> + <a class=\"trait\" href=\"num_traits/identities/trait.Zero.html\" title=\"trait num_traits::identities::Zero\">Zero</a> + <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedMul.html\" title=\"trait nalgebra::ClosedMul\">ClosedMul</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedAdd.html\" title=\"trait nalgebra::ClosedAdd\">ClosedAdd</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;T, D, D&gt;,</span>"]],
"num_bigint":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;T&gt; for <a class=\"struct\" href=\"num_bigint/struct.BigInt.html\" title=\"struct num_bigint::BigInt\">BigInt</a><span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"num_bigint/struct.BigInt.html\" title=\"struct num_bigint::BigInt\">BigInt</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;T, Output = <a class=\"struct\" href=\"num_bigint/struct.BigInt.html\" title=\"struct num_bigint::BigInt\">BigInt</a>&gt;,</span>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;T&gt; for <a class=\"struct\" href=\"num_bigint/struct.BigUint.html\" title=\"struct num_bigint::BigUint\">BigUint</a><span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"num_bigint/struct.BigUint.html\" title=\"struct num_bigint::BigUint\">BigUint</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;T, Output = <a class=\"struct\" href=\"num_bigint/struct.BigUint.html\" title=\"struct num_bigint::BigUint\">BigUint</a>&gt;,</span>"]],
"num_complex":[["impl&lt;T:&nbsp;<a class=\"trait\" href=\"num_traits/trait.Num.html\" title=\"trait num_traits::Num\">Num</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;<a class=\"struct\" href=\"num_complex/struct.Complex.html\" title=\"struct num_complex::Complex\">Complex</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"num_complex/struct.Complex.html\" title=\"struct num_complex::Complex\">Complex</a>&lt;T&gt;"],["impl&lt;'a, T:&nbsp;'a + <a class=\"trait\" href=\"num_traits/trait.Num.html\" title=\"trait num_traits::Num\">Num</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;&amp;'a <a class=\"struct\" href=\"num_complex/struct.Complex.html\" title=\"struct num_complex::Complex\">Complex</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"num_complex/struct.Complex.html\" title=\"struct num_complex::Complex\">Complex</a>&lt;T&gt;"]],
"num_rational":[["impl&lt;T:&nbsp;<a class=\"trait\" href=\"num_integer/trait.Integer.html\" title=\"trait num_integer::Integer\">Integer</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;<a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;"],["impl&lt;'a, T:&nbsp;<a class=\"trait\" href=\"num_integer/trait.Integer.html\" title=\"trait num_integer::Integer\">Integer</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/iter/traits/accum/trait.Product.html\" title=\"trait core::iter::traits::accum::Product\">Product</a>&lt;&amp;'a <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;"]],
"sp_std":[],
"wasmtime_environ":[]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()