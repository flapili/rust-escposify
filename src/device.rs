use std::fs;
use std::io;
use std::net;
use std::path;
use std::time::Duration;

use rusb;
use rusb::UsbContext;

#[derive(Debug)]
pub struct Usb {
    _vendor_id: u16,
    _product_id: u16,
    _interface: u8,
    _endpoint_in_address: u8,
    _endpoint_out_address: u8,
    _timeout: Duration,
    handle: rusb::DeviceHandle<rusb::Context>,
}
pub struct Serial {}

#[derive(Debug)]
pub struct Network {
    _host: String,
    _port: u16,
    stream: net::TcpStream,
}

impl Usb {
    pub fn new(
        vendor_id: u16,
        product_id: u16,
        interface: u8,
        endpoint_in_address: u8,
        endpoint_out_address: u8,
        timeout: Duration,
    ) -> Result<Usb, rusb::Error> {
        let context = rusb::Context::new()?;

        let device = context
            .devices()?
            .iter()
            .find(|device| {
                let desc = device.device_descriptor().unwrap();
                desc.vendor_id() == vendor_id && desc.product_id() == product_id
            })
            .ok_or(rusb::Error::NotFound)?;

        let mut handle = device.open()?;
        
        handle.set_auto_detach_kernel_driver(true).unwrap_or_default();
        handle.claim_interface(interface)?;
        Ok(Usb {
            _vendor_id: vendor_id,
            _product_id: product_id,
            _interface: interface,
            _endpoint_in_address: endpoint_in_address,
            _endpoint_out_address: endpoint_out_address,
            _timeout: timeout,
            handle,
        })
    }
}

impl io::Write for Usb {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.handle.write_bulk(self._endpoint_out_address, buf, self._timeout) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Network {
    pub fn new(host: &str, port: u16) -> io::Result<Network> {
        let stream = net::TcpStream::connect((host, port))?;
        Ok(Network {
            _host: host.to_string(),
            _port: port,
            stream,
        })
    }
}

impl io::Write for Network {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

/// File device that can be written to.

#[derive(Debug)]
pub struct File<W> {
    fobj: W,
}

impl<W: io::Write> File<W> {
    pub fn from_path<P: AsRef<path::Path> + ToString>(path: P) -> io::Result<File<fs::File>> {
        let fobj = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)?;
        Ok(File { fobj })
    }

    /// Create a device::File from a [std::io::Write].
    /// # Example
    /// ```rust
    /// use std::fs::File;
    /// use tempfile::NamedTempFileOptions;
    ///
    /// let tempf = NamedTempFileOptions::new().create().unwrap();
    /// let fobj = File::options().append(true).open(tempf.path()).unwrap();
    /// let file = escposify::device::File::from(fobj);
    /// ```
    pub fn from(fobj: W) -> File<W> {
        File { fobj }
    }
}

impl<W: io::Write> io::Write for File<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.fobj.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.fobj.flush()
    }
}
