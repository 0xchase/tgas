# TGAs

Do TGAs even work?

#### Statistical Methods

- [ ] **Pattern-Based Scanning (2015)** – learns the most common bit-patterns in a seed set and fixes them recursively to generate candidates. [Paper](https://doi.org/10.1109/ARES.2015.140)  
- [/] **Entropy/IP (2016)** – measures nybble-level entropy and builds a Bayesian model to sample new addresses matching observed statistics. [GitHub](https://github.com/akamai/entropy-ip)  
- [x] **6Gen (2017)** – clusters seed addresses by Hamming distance and outputs unobserved neighbors in the densest clusters. [Paper](https://doi.org/10.1145/3131365.3131382)  
- [x] **6Graph (2022)** – constructs a co-occurrence graph of address segments and recombines frequent subgraphs into new addresses. [Paper](https://doi.org/10.1016/j.comnet.2021.108666)  
- [x] **6Forest (2022)** – builds multiple space-partitioning trees (an ensemble) to cover diverse seed patterns before scanning. [Paper](https://doi.org/10.1109/INFOCOM.2022.9767014)  
- [.] **DET (2022)** – splits on the highest-entropy bits in the seed set to generate candidates with maximal variability. [Paper](https://doi.org/10.1109/TNET.2022.9678456)  
- [ ] **HMap6 (2023)** – merges agglomerative and divisive clustering outputs to capture both coarse and fine patterns. [Paper](https://doi.org/10.1109/INFOCOM.2023.10188415)  
- [ ] **AddrMiner (2022)** – transfers learned patterns across different prefixes and data sources to expand hitlists. [USENIX ATC ’22](https://www.usenix.org/conference/atc22/presentation/song)  

#### Machine Learning Methods

- [x] **6GCVAE (2020)** – trains a gated-CNN variational autoencoder on seed addresses and samples from its latent space. [Springer](https://link.springer.com/chapter/10.1007/978-3-030-50420-5_2)  
- [/] **6VecLM (2021)** – treats address blocks as “tokens” in a Transformer language model to predict new sequences. [Preprint](https://arxiv.org/abs/2107.08506)  
- [/] **6GAN (2021)** – uses clustered GANs with an alias-aware reward to generate pattern-specific addresses. [Paper](https://doi.org/10.1109/INFOCOM.2021.9452070)  
- [ ] **6MCBLM (2022)** – applies multi-scale CNNs plus BiLSTM to learn and generate addresses from block sequences. [Preprint](https://arxiv.org/abs/2211.12345)  
- [ ] **AGVCA (2023)** – uses a conditional VAE with context tags (e.g., prefix type) to steer generation. [Preprint](https://arxiv.org/abs/2305.01234)  
- [ ] **6Former (2023)** – tokenizes at half-nibble granularity and uses a Transformer to capture fine-grained address patterns. [Paper](https://doi.org/10.1109/ISCC.2023.10248413)  
- [ ] **6Diffusion (2024)** – applies a diffusion model on noisy address vectors and reverses noise to sample new targets. [Preprint](https://arxiv.org/abs/2412.19243)  

### Dynamic TGAs

#### Statistical Methods

- [.] **6Tree (2019)** – builds a hierarchical space tree over seeds and dynamically drills into branches with probe feedback. [Paper](https://doi.org/10.1016/j.comnet.2019.09.012)  
- [.] **6Scan (2023)** – divides the space into regions, encodes responsive areas, and continuously updates its target list. [Paper](https://doi.org/10.1109/TON.2023.10146589)  

#### Machine Learning Methods

- [ ] **6Hit (2021)** – frames scanning as an RL problem, rewarding probes that yield hits to guide future selections. [Paper](https://doi.org/10.1109/ICC.2021.9448749)  
- [ ] **6Rover (2024)** – uses an RL agent to explore “unseeded” gaps by rewarding discovery of novel active addresses. [Preprint](https://arxiv.org/abs/2401.07081)  
- [ ] **6SENSE (2024)** – integrates RL-based prefix selection, LSTM subnet prediction, and heuristic IID generation with live feedback. [USENIX Sec ’24](https://www.usenix.org/conference/usenixsecurity24/presentation/williams)
