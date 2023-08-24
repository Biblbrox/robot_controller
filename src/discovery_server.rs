pub mod discovery_server {
    use std::ffi::c_void;
    use std::mem::transmute;
    use std::ops::Deref;
    use std::ptr::{null, null_mut};
    use log::{debug, info};
    use crate::discovery_server_impl::{__u_char, __uint8_t, is_discovery_running_impl, on_participant_discovery_callback_t, ParticipantData, ParticipantData__bindgen_ty_1, ReaderData, register_on_participant_data, register_on_reader_data, register_on_writer_data, rmw_transport, run_discovery_server_impl, size_t, stop_discovery_server_impl, WriterData};

    //type Callback = fn(participant_data: ParticipantData);

    pub fn run_discovery_server<ParticipantFunc: Fn(ParticipantData), ReaderFunc: Fn(ReaderData), WriterFunc: Fn(WriterData)>(
        domain_id: u32,
        on_participant_discovery: ParticipantFunc,
        on_reader_discovery: ReaderFunc,
        on_writer_discovery: WriterFunc,
    ) {
        debug!("run_discovery_server");
        unsafe extern "C" fn wrapper_participant<F: Fn(ParticipantData)>(participant_data: ParticipantData, ctx: *mut ::std::os::raw::c_void) {
            (*(ctx as *const F))(participant_data);
        }
        unsafe extern "C" fn wrapper_reader<F: Fn(ReaderData)>(reader_data: ReaderData, ctx: *mut ::std::os::raw::c_void) {
            (*(ctx as *const F))(reader_data);
        }
        unsafe extern "C" fn wrapper_writer<F: Fn(WriterData)>(writer_data: WriterData, ctx: *mut ::std::os::raw::c_void) {
            (*(ctx as *const F))(writer_data);
        }

        unsafe {
            register_on_participant_data(&on_participant_discovery as *const ParticipantFunc as *mut ::std::os::raw::c_void);
            register_on_reader_data(&on_reader_discovery as *const ReaderFunc as *mut ::std::os::raw::c_void);
            register_on_writer_data(&on_writer_discovery as *const WriterFunc as *mut ::std::os::raw::c_void);
            run_discovery_server_impl(domain_id, Some(wrapper_participant::<ParticipantFunc>), Some(wrapper_reader::<ReaderFunc>), Some(wrapper_writer::<WriterFunc>));
        }
    }

    pub fn stop_discovery_server(domain_id: u32) {
        debug!("stop_discovery_server");
        unsafe {
            stop_discovery_server_impl(domain_id);
        }
    }

    pub fn is_discovery_running(domain_id: u32) -> bool {
        debug!("is_discovery_running");
        unsafe {
            return is_discovery_running_impl(domain_id) == 1;
        }
    }
}