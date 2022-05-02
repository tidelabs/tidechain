(function() {var implementors = {};
implementors["libp2p_core"] = [];
implementors["libp2p_deflate"] = [{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;C&gt; for <a class=\"struct\" href=\"libp2p_deflate/struct.DeflateConfig.html\" title=\"struct libp2p_deflate::DeflateConfig\">DeflateConfig</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a>,&nbsp;</span>","synthetic":false,"types":["libp2p_deflate::DeflateConfig"]}];
implementors["libp2p_floodsub"] = [{"text":"impl&lt;TSocket&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;TSocket&gt; for <a class=\"struct\" href=\"libp2p_floodsub/protocol/struct.FloodsubRpc.html\" title=\"struct libp2p_floodsub::protocol::FloodsubRpc\">FloodsubRpc</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TSocket: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_floodsub::protocol::FloodsubRpc"]}];
implementors["libp2p_gossipsub"] = [{"text":"impl&lt;TSocket&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;TSocket&gt; for <a class=\"struct\" href=\"libp2p_gossipsub/protocol/struct.ProtocolConfig.html\" title=\"struct libp2p_gossipsub::protocol::ProtocolConfig\">ProtocolConfig</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TSocket: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_gossipsub::protocol::ProtocolConfig"]}];
implementors["libp2p_kad"] = [{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;C&gt; for <a class=\"struct\" href=\"libp2p_kad/protocol/struct.KademliaProtocolConfig.html\" title=\"struct libp2p_kad::protocol::KademliaProtocolConfig\">KademliaProtocolConfig</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":false,"types":["libp2p_kad::protocol::KademliaProtocolConfig"]}];
implementors["libp2p_mplex"] = [{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;C&gt; for <a class=\"struct\" href=\"libp2p_mplex/struct.MplexConfig.html\" title=\"struct libp2p_mplex::MplexConfig\">MplexConfig</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":false,"types":["libp2p_mplex::config::MplexConfig"]}];
implementors["libp2p_noise"] = [{"text":"impl&lt;T, C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;T&gt; for <a class=\"struct\" href=\"libp2p_noise/struct.NoiseConfig.html\" title=\"struct libp2p_noise::NoiseConfig\">NoiseConfig</a>&lt;<a class=\"enum\" href=\"libp2p_noise/enum.IX.html\" title=\"enum libp2p_noise::IX\">IX</a>, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"libp2p_noise/struct.NoiseConfig.html\" title=\"struct libp2p_noise::NoiseConfig\">NoiseConfig</a>&lt;<a class=\"enum\" href=\"libp2p_noise/enum.IX.html\" title=\"enum libp2p_noise::IX\">IX</a>, C&gt;: <a class=\"trait\" href=\"libp2p_core/upgrade/trait.UpgradeInfo.html\" title=\"trait libp2p_core::upgrade::UpgradeInfo\">UpgradeInfo</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"libp2p_noise/trait.Protocol.html\" title=\"trait libp2p_noise::Protocol\">Protocol</a>&lt;C&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.slice.html\">[</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.u8.html\">u8</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.slice.html\">]</a>&gt; + <a class=\"trait\" href=\"zeroize/trait.Zeroize.html\" title=\"trait zeroize::Zeroize\">Zeroize</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_noise::NoiseConfig"]},{"text":"impl&lt;T, C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;T&gt; for <a class=\"struct\" href=\"libp2p_noise/struct.NoiseConfig.html\" title=\"struct libp2p_noise::NoiseConfig\">NoiseConfig</a>&lt;<a class=\"enum\" href=\"libp2p_noise/enum.XX.html\" title=\"enum libp2p_noise::XX\">XX</a>, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"libp2p_noise/struct.NoiseConfig.html\" title=\"struct libp2p_noise::NoiseConfig\">NoiseConfig</a>&lt;<a class=\"enum\" href=\"libp2p_noise/enum.XX.html\" title=\"enum libp2p_noise::XX\">XX</a>, C&gt;: <a class=\"trait\" href=\"libp2p_core/upgrade/trait.UpgradeInfo.html\" title=\"trait libp2p_core::upgrade::UpgradeInfo\">UpgradeInfo</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"libp2p_noise/trait.Protocol.html\" title=\"trait libp2p_noise::Protocol\">Protocol</a>&lt;C&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.slice.html\">[</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.u8.html\">u8</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.slice.html\">]</a>&gt; + <a class=\"trait\" href=\"zeroize/trait.Zeroize.html\" title=\"trait zeroize::Zeroize\">Zeroize</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_noise::NoiseConfig"]},{"text":"impl&lt;T, C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;T&gt; for <a class=\"struct\" href=\"libp2p_noise/struct.NoiseConfig.html\" title=\"struct libp2p_noise::NoiseConfig\">NoiseConfig</a>&lt;<a class=\"enum\" href=\"libp2p_noise/enum.IK.html\" title=\"enum libp2p_noise::IK\">IK</a>, C, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.tuple.html\">(</a><a class=\"struct\" href=\"libp2p_noise/struct.PublicKey.html\" title=\"struct libp2p_noise::PublicKey\">PublicKey</a>&lt;C&gt;, <a class=\"enum\" href=\"libp2p_core/identity/enum.PublicKey.html\" title=\"enum libp2p_core::identity::PublicKey\">PublicKey</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.tuple.html\">)</a>&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"libp2p_noise/struct.NoiseConfig.html\" title=\"struct libp2p_noise::NoiseConfig\">NoiseConfig</a>&lt;<a class=\"enum\" href=\"libp2p_noise/enum.IK.html\" title=\"enum libp2p_noise::IK\">IK</a>, C, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.tuple.html\">(</a><a class=\"struct\" href=\"libp2p_noise/struct.PublicKey.html\" title=\"struct libp2p_noise::PublicKey\">PublicKey</a>&lt;C&gt;, <a class=\"enum\" href=\"libp2p_core/identity/enum.PublicKey.html\" title=\"enum libp2p_core::identity::PublicKey\">PublicKey</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.tuple.html\">)</a>&gt;: <a class=\"trait\" href=\"libp2p_core/upgrade/trait.UpgradeInfo.html\" title=\"trait libp2p_core::upgrade::UpgradeInfo\">UpgradeInfo</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"libp2p_noise/trait.Protocol.html\" title=\"trait libp2p_noise::Protocol\">Protocol</a>&lt;C&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.slice.html\">[</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.u8.html\">u8</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.slice.html\">]</a>&gt; + <a class=\"trait\" href=\"zeroize/trait.Zeroize.html\" title=\"trait zeroize::Zeroize\">Zeroize</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_noise::NoiseConfig"]},{"text":"impl&lt;T, P, C, R&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;T&gt; for <a class=\"struct\" href=\"libp2p_noise/struct.NoiseAuthenticated.html\" title=\"struct libp2p_noise::NoiseAuthenticated\">NoiseAuthenticated</a>&lt;P, C, R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"libp2p_noise/struct.NoiseConfig.html\" title=\"struct libp2p_noise::NoiseConfig\">NoiseConfig</a>&lt;P, C, R&gt;: <a class=\"trait\" href=\"libp2p_core/upgrade/trait.UpgradeInfo.html\" title=\"trait libp2p_core::upgrade::UpgradeInfo\">UpgradeInfo</a> + <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;T, Output = <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.tuple.html\">(</a><a class=\"enum\" href=\"libp2p_noise/handshake/enum.RemoteIdentity.html\" title=\"enum libp2p_noise::handshake::RemoteIdentity\">RemoteIdentity</a>&lt;C&gt;, <a class=\"struct\" href=\"libp2p_noise/struct.NoiseOutput.html\" title=\"struct libp2p_noise::NoiseOutput\">NoiseOutput</a>&lt;T&gt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.tuple.html\">)</a>, Error = <a class=\"enum\" href=\"libp2p_noise/enum.NoiseError.html\" title=\"enum libp2p_noise::NoiseError\">NoiseError</a>&gt; + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;<a class=\"struct\" href=\"libp2p_noise/struct.NoiseConfig.html\" title=\"struct libp2p_noise::NoiseConfig\">NoiseConfig</a>&lt;P, C, R&gt; as <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;T&gt;&gt;::<a class=\"associatedtype\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html#associatedtype.Future\" title=\"type libp2p_core::upgrade::OutboundUpgrade::Future\">Future</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"libp2p_noise/trait.Protocol.html\" title=\"trait libp2p_noise::Protocol\">Protocol</a>&lt;C&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.slice.html\">[</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.u8.html\">u8</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.slice.html\">]</a>&gt; + <a class=\"trait\" href=\"zeroize/trait.Zeroize.html\" title=\"trait zeroize::Zeroize\">Zeroize</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_noise::NoiseAuthenticated"]}];
implementors["libp2p_plaintext"] = [{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;C&gt; for <a class=\"struct\" href=\"libp2p_plaintext/struct.PlainText1Config.html\" title=\"struct libp2p_plaintext::PlainText1Config\">PlainText1Config</a>","synthetic":false,"types":["libp2p_plaintext::PlainText1Config"]},{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;C&gt; for <a class=\"struct\" href=\"libp2p_plaintext/struct.PlainText2Config.html\" title=\"struct libp2p_plaintext::PlainText2Config\">PlainText2Config</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_plaintext::PlainText2Config"]}];
implementors["libp2p_request_response"] = [{"text":"impl&lt;TCodec&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;<a class=\"struct\" href=\"multistream_select/negotiated/struct.Negotiated.html\" title=\"struct multistream_select::negotiated::Negotiated\">Negotiated</a>&lt;<a class=\"struct\" href=\"libp2p_core/muxing/struct.SubstreamRef.html\" title=\"struct libp2p_core::muxing::SubstreamRef\">SubstreamRef</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.59.0/alloc/sync/struct.Arc.html\" title=\"struct alloc::sync::Arc\">Arc</a>&lt;<a class=\"struct\" href=\"libp2p_core/muxing/struct.StreamMuxerBox.html\" title=\"struct libp2p_core::muxing::StreamMuxerBox\">StreamMuxerBox</a>&gt;&gt;&gt;&gt; for <a class=\"struct\" href=\"libp2p_request_response/handler/struct.RequestProtocol.html\" title=\"struct libp2p_request_response::handler::RequestProtocol\">RequestProtocol</a>&lt;TCodec&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TCodec: <a class=\"trait\" href=\"libp2p_request_response/codec/trait.RequestResponseCodec.html\" title=\"trait libp2p_request_response::codec::RequestResponseCodec\">RequestResponseCodec</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_request_response::handler::protocol::RequestProtocol"]}];
implementors["libp2p_swarm"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"libp2p_swarm/handler/trait.OutboundUpgradeSend.html\" title=\"trait libp2p_swarm::handler::OutboundUpgradeSend\">OutboundUpgradeSend</a>&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;<a class=\"struct\" href=\"multistream_select/negotiated/struct.Negotiated.html\" title=\"struct multistream_select::negotiated::Negotiated\">Negotiated</a>&lt;<a class=\"struct\" href=\"libp2p_core/muxing/struct.SubstreamRef.html\" title=\"struct libp2p_core::muxing::SubstreamRef\">SubstreamRef</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.59.0/alloc/sync/struct.Arc.html\" title=\"struct alloc::sync::Arc\">Arc</a>&lt;<a class=\"struct\" href=\"libp2p_core/muxing/struct.StreamMuxerBox.html\" title=\"struct libp2p_core::muxing::StreamMuxerBox\">StreamMuxerBox</a>&gt;&gt;&gt;&gt; for <a class=\"struct\" href=\"libp2p_swarm/handler/struct.SendWrapper.html\" title=\"struct libp2p_swarm::handler::SendWrapper\">SendWrapper</a>&lt;T&gt;","synthetic":false,"types":["libp2p_swarm::upgrade::SendWrapper"]}];
implementors["libp2p_yamux"] = [{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;C&gt; for <a class=\"struct\" href=\"libp2p_yamux/struct.YamuxConfig.html\" title=\"struct libp2p_yamux::YamuxConfig\">YamuxConfig</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_yamux::YamuxConfig"]},{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"libp2p_core/upgrade/trait.OutboundUpgrade.html\" title=\"trait libp2p_core::upgrade::OutboundUpgrade\">OutboundUpgrade</a>&lt;C&gt; for <a class=\"struct\" href=\"libp2p_yamux/struct.YamuxLocalConfig.html\" title=\"struct libp2p_yamux::YamuxLocalConfig\">YamuxLocalConfig</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_yamux::YamuxLocalConfig"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()