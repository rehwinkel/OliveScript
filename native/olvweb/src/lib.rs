use httparse;
use olvnative::{Object, RuntimeError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;

macro_rules! rc {
    ($e: expr) => {
        Rc::new(RefCell::new($e))
    };
}

macro_rules! generic_err {
    ($e: expr) => {
        $e.map_err(|e| RuntimeError::Error(format!("{}", e)))?
    };
}

type RcObject = Rc<RefCell<Object>>;

fn to_ptr<T>(obj: T) -> *mut usize {
    let listener = Box::new(obj);
    Box::into_raw(listener) as *mut usize
}

#[no_mangle]
pub unsafe extern "C" fn n_bind(args: Box<Vec<RcObject>>) -> Result<RcObject, RuntimeError> {
    if let Object::Str(addr) = &*args[0].borrow() {
        let listener = generic_err!(TcpListener::bind(addr));
        Ok(rc!(Object::Pointer(to_ptr(listener))))
    } else {
        Err(RuntimeError::TypeError)
    }
}

#[no_mangle]
pub unsafe extern "C" fn n_recv(args: Box<Vec<RcObject>>) -> Result<RcObject, RuntimeError> {
    if let Object::Pointer(ptr) = &*args[0].borrow() {
        let listener_ptr: *mut TcpListener = std::mem::transmute(*ptr);
        let listener = &*listener_ptr;
        let (mut client, addr) = generic_err!(listener.accept());

        let mut data: Vec<u8> = Vec::new();
        loop {
            let mut buffer: [u8; 20] = [0; 20];
            let read_bytes = generic_err!(client.read(&mut buffer));
            data.append(&mut (buffer[0..read_bytes].to_vec()));
            if let Some(mut result) = parse(&data, addr.to_string())? {
                result.insert(String::from("client"), rc!(Object::Pointer(to_ptr(client))));
                return Ok(rc!(Object::Bendy(result)));
            }
        }
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn parse(content: &[u8], addr: String) -> Result<Option<HashMap<String, RcObject>>, RuntimeError> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    if let Ok(r) = req.parse(content) {
        if r.is_partial() {
            return Ok(None);
        }
    } else {
        return Ok(None);
    }
    let strval = match String::from_utf8(content.to_vec()) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    let parts: Vec<&str> = strval.split("\r\n\r\n").collect();
    if parts.len() != 2 {
        return Ok(None);
    }
    let content: String = String::from(parts[1]);
    let path: String = String::from(req.path.unwrap());
    let method: String = String::from(req.method.unwrap());
    let version: String = req.version.unwrap().to_string();
    let mut headers = HashMap::new();

    for header in req.headers {
        let name: String = String::from(header.name);
        headers.insert(
            name.to_ascii_lowercase(),
            rc!(Object::Str(generic_err!(String::from_utf8(
                header.value.to_vec()
            )))),
        );
    }

    let content_len = if let Some(content_len_obj) = headers.get(&String::from("content-length")) {
        if let Object::Str(content_len_str) = &*content_len_obj.borrow() {
            content_len_str.parse::<usize>().unwrap()
        } else {
            return Err(RuntimeError::TypeError)
        }
    } else {
        0
    };

    if content.len() != content_len {
        return Ok(None);
    }

    let mut map = HashMap::new();
    map.insert(String::from("content"), rc!(Object::Str(content)));
    map.insert(String::from("path"), rc!(Object::Str(path)));
    map.insert(String::from("version"), rc!(Object::Str(version)));
    map.insert(String::from("method"), rc!(Object::Str(method)));
    map.insert(String::from("headers"), rc!(Object::Bendy(headers)));
    map.insert(String::from("addr"), rc!(Object::Str(addr)));
    Ok(Some(map))
}

#[no_mangle]
pub unsafe extern "C" fn n_send(args: Box<Vec<RcObject>>) -> Result<RcObject, RuntimeError> {
    if let Object::Pointer(ptr) = &*args[0].borrow() {
        if let Object::Str(data) = &*args[1].borrow() {
            let stream_ptr: *mut TcpStream = std::mem::transmute(*ptr);
            let mut stream = &*stream_ptr;
            generic_err!(stream.write(create_res(data).as_bytes()));
            Ok(rc!(Object::None))
        } else {
            Err(RuntimeError::TypeError)
        }
    } else {
    Err(RuntimeError::TypeError)
    }
}


fn create_res(content: &String) -> String {
    format!("HTTP/1.1 200 OK\nConnection: keep-alive\nContent-Length: {}\nDate: Sat, 29 Feb 2020 14:14:31 GMT\n\n{}", content.len(), content)
}

