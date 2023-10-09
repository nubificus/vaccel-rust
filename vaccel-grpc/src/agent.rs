// This file is generated by rust-protobuf 3.3.0. Do not edit
// .proto file is parsed by pure
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_results)]
#![allow(unused_mut)]

//! Generated file from `agent.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_3_3_0;

// @@protoc_insertion_point(message:vaccel.VaccelEmpty)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct VaccelEmpty {
    // special fields
    // @@protoc_insertion_point(special_field:vaccel.VaccelEmpty.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a VaccelEmpty {
    fn default() -> &'a VaccelEmpty {
        <VaccelEmpty as ::protobuf::Message>::default_instance()
    }
}

impl VaccelEmpty {
    pub fn new() -> VaccelEmpty {
        ::std::default::Default::default()
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(0);
        let mut oneofs = ::std::vec::Vec::with_capacity(0);
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<VaccelEmpty>(
            "VaccelEmpty",
            fields,
            oneofs,
        )
    }
}

impl ::protobuf::Message for VaccelEmpty {
    const NAME: &'static str = "VaccelEmpty";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> VaccelEmpty {
        VaccelEmpty::new()
    }

    fn clear(&mut self) {
        self.special_fields.clear();
    }

    fn default_instance() -> &'static VaccelEmpty {
        static instance: VaccelEmpty = VaccelEmpty {
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

impl ::protobuf::MessageFull for VaccelEmpty {
    fn descriptor() -> ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::Lazy::new();
        descriptor.get(|| file_descriptor().message_by_package_relative_name("VaccelEmpty").unwrap()).clone()
    }
}

impl ::std::fmt::Display for VaccelEmpty {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for VaccelEmpty {
    type RuntimeType = ::protobuf::reflect::rt::RuntimeTypeMessage<Self>;
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0bagent.proto\x12\x06vaccel\x1a\rsession.proto\x1a\x0fresources.prot\
    o\x1a\x0bimage.proto\x1a\x10tensorflow.proto\x1a\x0btorch.proto\x1a\x0bg\
    enop.proto\x1a\x0fprofiling.proto\"\r\n\x0bVaccelEmpty2\xf1\x08\n\x0bVac\
    celAgent\x12L\n\rCreateSession\x12\x1c.vaccel.CreateSessionRequest\x1a\
    \x1d.vaccel.CreateSessionResponse\x12B\n\rUpdateSession\x12\x1c.vaccel.U\
    pdateSessionRequest\x1a\x13.vaccel.VaccelEmpty\x12D\n\x0eDestroySession\
    \x12\x1d.vaccel.DestroySessionRequest\x1a\x13.vaccel.VaccelEmpty\x12O\n\
    \x0eCreateResource\x12\x1d.vaccel.CreateResourceRequest\x1a\x1e.vaccel.C\
    reateResourceResponse\x12H\n\x10RegisterResource\x12\x1f.vaccel.Register\
    ResourceRequest\x1a\x13.vaccel.VaccelEmpty\x12L\n\x12UnregisterResource\
    \x12!.vaccel.UnregisterResourceRequest\x1a\x13.vaccel.VaccelEmpty\x12F\n\
    \x0fDestroyResource\x12\x1e.vaccel.DestroyResourceRequest\x1a\x13.vaccel\
    .VaccelEmpty\x12^\n\x13ImageClassification\x12\".vaccel.ImageClassificat\
    ionRequest\x1a#.vaccel.ImageClassificationResponse\x12^\n\x13TensorflowM\
    odelLoad\x12\".vaccel.TensorflowModelLoadRequest\x1a#.vaccel.TensorflowM\
    odelLoadResponse\x12d\n\x15TensorflowModelUnload\x12$.vaccel.TensorflowM\
    odelUnloadRequest\x1a%.vaccel.TensorflowModelUnloadResponse\x12[\n\x12Te\
    nsorflowModelRun\x12!.vaccel.TensorflowModelRunRequest\x1a\".vaccel.Tens\
    orflowModelRunResponse\x12^\n\x13TorchJitloadForward\x12\".vaccel.TorchJ\
    itloadForwardRequest\x1a#.vaccel.TorchJitloadForwardResponse\x124\n\x05G\
    enop\x12\x14.vaccel.GenopRequest\x1a\x15.vaccel.GenopResponse\x12@\n\tGe\
    tTimers\x12\x18.vaccel.ProfilingRequest\x1a\x19.vaccel.ProfilingResponse\
    b\x06proto3\
";

/// `FileDescriptorProto` object which was a source for this generated file
fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    static file_descriptor_proto_lazy: ::protobuf::rt::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::Lazy::new();
    file_descriptor_proto_lazy.get(|| {
        ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
    })
}

/// `FileDescriptor` object which allows dynamic access to files
pub fn file_descriptor() -> &'static ::protobuf::reflect::FileDescriptor {
    static generated_file_descriptor_lazy: ::protobuf::rt::Lazy<::protobuf::reflect::GeneratedFileDescriptor> = ::protobuf::rt::Lazy::new();
    static file_descriptor: ::protobuf::rt::Lazy<::protobuf::reflect::FileDescriptor> = ::protobuf::rt::Lazy::new();
    file_descriptor.get(|| {
        let generated_file_descriptor = generated_file_descriptor_lazy.get(|| {
            let mut deps = ::std::vec::Vec::with_capacity(7);
            deps.push(super::session::file_descriptor().clone());
            deps.push(super::resources::file_descriptor().clone());
            deps.push(super::image::file_descriptor().clone());
            deps.push(super::tensorflow::file_descriptor().clone());
            deps.push(super::torch::file_descriptor().clone());
            deps.push(super::genop::file_descriptor().clone());
            deps.push(super::profiling::file_descriptor().clone());
            let mut messages = ::std::vec::Vec::with_capacity(1);
            messages.push(VaccelEmpty::generated_message_descriptor_data());
            let mut enums = ::std::vec::Vec::with_capacity(0);
            ::protobuf::reflect::GeneratedFileDescriptor::new_generated(
                file_descriptor_proto(),
                deps,
                messages,
                enums,
            )
        });
        ::protobuf::reflect::FileDescriptor::new_generated_2(generated_file_descriptor)
    })
}
