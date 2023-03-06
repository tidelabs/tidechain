(function() {var implementors = {
"bytes":[["impl&lt;B:&nbsp;<a class=\"trait\" href=\"bytes/buf/trait.Buf.html\" title=\"trait bytes::buf::Buf\">Buf</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> for <a class=\"struct\" href=\"bytes/buf/struct.Reader.html\" title=\"struct bytes::buf::Reader\">Reader</a>&lt;B&gt;"]],
"either":[["impl&lt;L, R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> for <a class=\"enum\" href=\"either/enum.Either.html\" title=\"enum either::Either\">Either</a>&lt;L, R&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a>,</span>"]],
"flate2":[["impl&lt;R:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> for <a class=\"struct\" href=\"flate2/struct.CrcReader.html\" title=\"struct flate2::CrcReader\">CrcReader</a>&lt;R&gt;"]],
"futures_lite":[["impl&lt;T:&nbsp;<a class=\"trait\" href=\"futures_io/if_std/trait.AsyncBufRead.html\" title=\"trait futures_io::if_std::AsyncBufRead\">AsyncBufRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> for <a class=\"struct\" href=\"futures_lite/io/struct.BlockOn.html\" title=\"struct futures_lite::io::BlockOn\">BlockOn</a>&lt;T&gt;"]],
"futures_util":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> for <a class=\"struct\" href=\"futures_util/io/struct.AllowStdIo.html\" title=\"struct futures_util::io::AllowStdIo\">AllowStdIo</a>&lt;T&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.67.1/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a>,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()