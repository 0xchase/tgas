# Notes

## Papers

- **Paper 1**: Tool paper
- **Paper 2**: TGAs don't generalize to clients
- **Paper 3**: Vulnerability identification scan but for IPv6
- **Paper 4**: IPFS for address discovery (cache content that indicates vulnerability)
- **Paper 5**: Solve the alias detection issue

## Ideas

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

- **analyze**: `analyze`, `visualize`
  - by default it will identify the file and suggest arguments/flags for the user to pass
  - **scan output**: parse any zmap or ipv6kit output
  - **tga model**: parse a tga model file, identify the tga, load the tga, print supported information about it
  - **address list**: parse an address list
  - metrics
  - tables
  - graphs
    - graph any property against another property
      - pre-calculate some properties to make them available
      - use property types (category/enum, float, int, etc) to decide graph format
      - line graph, bar chart, pie chart, scatter plot, heatmap
  - visualizations
  - classify addresses
- **measurements**: `measure`
  - measure the bandwidth of an interface
  - measure cpu/memory usage
- **scanners**: `scan`
  - support stateful, stateless, PF_RING accelerated stateless, and application layer
  - support as many zmap options as possible
  - live detection
  - alias detection
  - routed detection
  - response from probed address or third-party
  - `--analysis` flag can be passed a list of analysis plugins
  - `--watch` and `--feedback` can show other things
  - *feedback*: live update any analysis
- **tgas**: `generate`, `train`, `run`
  - some in Rust
  - some in python over `pyo3`
  - *feedback*: live update any analysis
- **data**: `download`
  - download common data sources
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
