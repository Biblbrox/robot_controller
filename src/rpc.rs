extern crate xmlrpc;

use xmlrpc::{Request, Value};

/*pub mod rpc {
    pub fn make_request(request: &str) {
        // The Python example server exports Python's `pow` method. Let's call it!
        let host = "localhost";
        let port = 11511;
        let url = "http://rmw_fastrtps_cpp/";

        let request = Request::new("get_node_names").arg(2).arg(8); // Compute 2**8

        let request_result = request.call_url(url);

        println!("Result: {:?}", request_result);

        let pow_result = request_result.unwrap();
        assert_eq!(pow_result, Value::Int(2i32.pow(8)));
    }
}*/