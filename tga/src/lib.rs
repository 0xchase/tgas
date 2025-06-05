mod entropy_ip;

use std::net::Ipv6Addr;

use entropy_ip::EntropyIpTga;


// generates new targets given a seed
/*
Don't use static and dynamic TGAs
- Pure algorithmic methods are a simple TGA type
- Training a model may output a TGA
- Dynamic TGAs transform a TGA into another TGA
*/

// Things for tgas to do
/*
- Transform a list of targets into a new list of targets
- Transform a list of targets into a model
  - Transform a model into a target iterator
- Only generate for part of the prefix space
- Should operate on byte slices of a fixed size

IpNetModel is an IPAddr iterator

  */

/// The struct that implements this trait will specify the settings for that TGA
pub trait TGA<const C: usize> {
    type Model: IpNetModel<C>;

    /// Build the model of the address space from a list of seeds
    fn build<T: IntoIterator<Item = [u8; C]>>(&self, seeds: T) -> Self::Model;
}

/// The struct that implements this trait will generate new targets from its model
pub trait IpNetModel<const C: usize> {
    fn generate(&self) -> [u8; C];
}


pub fn test(count: usize) {
    // Sample IPv6 addresses inspired by patterns discussed in the paper.
    // Some have structured prefixes, some have similar interface IDs.
    let seed_ips: Vec<[u8; 16]> = vec![
        Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0001, 0, 0, 0, 0x0001).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0001, 0, 0, 0, 0x0002).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0002, 0, 0, 0, 0x0001).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0002, 0, 0, 0, 0x0002).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000a, 0, 0, 0, 0x000a).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000a, 0, 0, 0, 0x000b).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000b, 0, 0, 0, 0x000a).octets(),
        // Add an address that's quite different to influence entropy
        Ipv6Addr::new(0x2001, 0x0db8, 0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6667).octets(),
    ];

    let tga = EntropyIpTga;
    let model = tga.build(seed_ips);

    // Generate new candidate addresses from the model.
    for i in 0..count {
        let generated_bytes = model.generate();
        let generated_ip = Ipv6Addr::from(generated_bytes);
        println!("  {}: {}", i + 1, generated_ip);
    }
}