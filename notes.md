# Notes

## Papers

- **Paper 1**: Tool paper
- **Paper 2**: TGAs don't generalize to clients
 - Assess the performance/accuracy tradeoff
- **Paper 3**: Vulnerability identification scan but for IPv6
- **Paper 4**: IPFS for address discovery (cache content that indicates vulnerability)
- **Paper 5**: Solve the alias detection issue

## Ideas

Plugin system
- Use a build.rs plus an inventory to auto-register plugins
- Users can `cargo add` a plugin and it will be auto-discovered by the installation

Consider an Arrow/Parquet IO schema?
Use OpenMetrics + Prometheus to remotely monitor jobs
Output to various apache arrow types like JSON, CSV, etc

Use cargo to generate markdown docs website.
Beautiful [ratatui](https://ratatui.rs/examples/apps/) user interface

Scan result:
- dispersion
- percent live, aliased, response from probed/third-party, etc
- address set entropy
- yield
- coverage (across addrs, prefixes, etc)
- classify addresses in some way

TGA results
- overlap w/training or input data
- duplicates output

System metrics:
- job count
- job progress
- job status
- global CPU/Memory usage
- global system properties (OS, CPU, memory, etc)
- output log

## Framework

Core Modules

- **view**
 - Simple TUI view for interactively browsing and searching a dataframe
 - View the schema files in various ways
- **analyze**
  - *formats*: ip list, scan output, tga model
  - *commands*:
    - counts: better print, work for CSV, count by subnets, count CSV property
    - dispersion: todo
    - entropy: better print, work for CSV
    - subnets
    - graph: zesplot, bar, line, pie, scatter, heatmap, categories, (property vs. property)
  - classify addresses
- **plot**
  - Use the `plotlars` crate to easily plot any `DataFrame`
- **filter/extract**
  - Modifying address lists or scan outputs
  - merge: merge and interweave two scans, or append two address lists
  - search: search for contents
- **measure**
  - measure the bandwidth of an interface
  - measure maximum scan speed
  - measure cpu/memory usage
- **locate**
  - locate localhost/self using various techniques
  - round-trip-time triangulation (pass three scans to `analyze`)
  - like `ip2trace` combine traceroute with offline database lookup for each hop
  - using BGP and WHOIS data to find the registered owner
- **lookup**
  - reverse DNS lookup
  - whois lookup
  - IP to ASN mapping using a service
- **scan**
  - support stateful, stateless, PF_RING accelerated stateless, and application layer
  - support as many zmap options as possible
  - traceroute
  - response from probed address or third-party
  - `--analysis` flag can be passed a list of analysis plugins
  - `--watch` and `--feedback` can show other things
  - *feedback*: live update any analysis
- **detect**
  - aliased, live, routed
- **tgas**: `generate`, `train`, `run`, `discover`
  - some in Rust
  - some in python over `pyo3`
  - *feedback*: live update any analysis
- **download**
  - download common data sources
  - BGP, DNS, CDNs, etc.
  - https://opendata.rapid7.com/
  - load from files
  - parse from arguments
- **server**: `serve`
  - starts tonic `grpc` server
  - bi-directional asynchronous streaming
  - `scanner` to host a scanning server
  - `bandwidth` to host a bandwidth test server

## CLI Frontend

- In basic mode it just launches a single task

## Python Scripting Frontend

- Script a certain scan/analysis with python

## Remote CLI Frontend

- In remote mode it will connect over gRPC for job management

## MCP Frontend

- Model context protocol for managing it

## Flutter Frontend

- Can connect to other packages over gRPC
- Can render images created by analyzer plugin
- Can queue available data sets
- Fancy interface for viewing job progress
- Graphic of the whole pipeline: can select plugin(s) to run at each stage
