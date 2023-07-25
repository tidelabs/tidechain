window.SIDEBAR_ITEMS = {"derive":[["CompactAs","Derive `parity_scale_codec::Compact` and `parity_scale_codec::CompactAs` for struct with single field."],["Decode","Derive `parity_scale_codec::Decode` and for struct and enum."],["Encode","Derive `parity_scale_codec::Encode` and `parity_scale_codec::EncodeLike` for struct and enum."],["MaxEncodedLen","Derive macro for [`MaxEncodedLen`][max_encoded_len::MaxEncodedLen]."]],"fn":[["decode_from_bytes","Decodes a given `T` from `Bytes`."],["decode_vec_with_len","Decode the vec (without a prepended len)."]],"struct":[["Compact","Compact-encoded variant of T. This is more space-efficient but less compute-efficient."],["CompactRef","Compact-encoded variant of &’a T. This is more space-efficient but less compute-efficient."],["DecodeFinished","A zero-sized type signifying that the decoding finished."],["Error","Error type."],["IoReader","Wrapper that implements Input for any `Read` type."],["OptionBool","Shim type because we can’t do a specialised implementation for `Option<bool>` directly."],["Ref","Reference wrapper that implement encode like any type that is encoded like its inner type."]],"trait":[["Codec","Trait that allows zero-copy read/write of value-references to/from slices in LE format."],["CompactAs","Allow foreign structs to be wrap in Compact"],["CompactLen","Something that can return the compact encoded length for a given value."],["ConstEncodedLen","Types that have a constant encoded length. This implies [`MaxEncodedLen`]."],["Decode","Trait that allows zero-copy read of value-references from slices in LE format."],["DecodeAll","Extension trait to [`Decode`] that ensures that the given input data is consumed completely while decoding."],["DecodeLength","Trait that allows the length of a collection to be read, without having to read and decode the entire elements."],["DecodeLimit","Extension trait to [`Decode`] for decoding with a maximum recursion depth."],["Encode","Trait that allows zero-copy write of value-references to slices in LE format."],["EncodeAppend","Trait that allows to append items to an encoded representation without decoding all previous added items."],["EncodeAsRef","Something that can be encoded as a reference."],["EncodeLike","A marker trait that tells the compiler that a type encode to the same representation as another type."],["FullCodec","Trait that bound `EncodeLike` along with `Codec`. Usefull for generic being used in function with `EncodeLike` parameters."],["FullEncode","Trait that bound `EncodeLike` along with `Encode`. Usefull for generic being used in function with `EncodeLike` parameters."],["HasCompact","Trait that tells you if a given type can be encoded/decoded in a compact way."],["Input","Trait that allows reading of data into a slice."],["Joiner","Trait to allow itself to be serialised into a value which can be extended by bytes."],["KeyedVec","Trait to allow itself to be serialised and prepended by a given slice."],["MaxEncodedLen","Items implementing `MaxEncodedLen` have a statically known maximum encoded size."],["Output","Trait that allows writing of data."],["WrapperTypeDecode","A marker trait for types that can be created solely from other decodable types."],["WrapperTypeEncode","A marker trait for types that wrap other encodable type."]]};