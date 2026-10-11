# Human-librarian validity packet

120 pairs. Label each pair in `labels.csv` (one row per uid) by the rubric in
`../rubric.md`, section "Labels": `relation` (same-topic, same-field, different-field),
`together` (together, apart), `scheme_decision` (correct, incorrect), `class_1_ok` and
`class_2_ok` (yes, no), and an optional one-line `reason`. Judge from the titles,
abstracts and class paths below only. Do not open `../sample.jsonl`, `../results.md`
or any benchmark result before you finish: they hold the metric's and the LLM judge's
view of the same pairs.

Annotator, date and any rubric questions go in `annotator.md`.

## 1. uid `3c13a90d86`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** LOONG: Online Time-Optimal Autonomous Flight for MAVs in Cluttered Environments

> Autonomous flight of micro air vehicles (MAVs) in unknown, cluttered environments remains challenging for time-critical missions due to conservative maneuvering strategies. This article presents an integrated planning and control framework for high-speed, time-optimal autonomous flight of MAVs in cluttered environments. In each replanning cycle (100 Hz), a time-optimal trajectory under polynomial presentation is generated as a reference, with the time-allocation process accelerated by imitation learning. Subsequently, a time-optimal model predictive contouring control (MPCC) incorporates safe flight corridor (SFC) constraints at variable horizon steps to enable aggressive yet safe maneuvering, while fully exploiting the MAV's dynamics. We validate the proposed framework extensively on a custom-built LiDAR-based MAV platform. Simulation results demonstrate superior aggressiveness …

Class path: b — spacetime › bb — time

**Work 2.** Future-Aware Flow Planning for Safe UAV Target Following

> UAV target following in cluttered environments requires anticipating target motion. Followers that use only the current target state can lag behind turns or choose blocked corridors. They may also trade safe near-horizon motion for lower tracking error. We propose a future-aware flow planning framework for state-informed UAV target following. Predicted target futures guide clean UAV trajectory generation through residual signals aligned with the planning horizon. Risk-scored repair of the executable prefix is embedded in the sampling loop. On fixed in-distribution (ID) and out-of-distribution (OOD) receding-horizon benchmarks, the planner improves the safety--tracking trade-off. It matches zero measured ID collision rate and achieves the highest ID safe-tracking time fraction. It also gives the lowest OOD macro-average collision rate and final tracking error among the compared methods. …

Class path: p — consciousness › pj — unconscious mind › pjd — id

## 2. uid `f79f47ace5`

Scheme: OpenAlex Topics

**Work 1.** Transient Synchronization Stability Analysis of SG-DFIG Parallel System Considering Complete LVRT Processes

> Although a large amount of work has been devoted to detailed electromagnetic transient simulation in analyzing transient synchronization stability (TSS) of hybrid systems containing renewable energy equipment and synchronous generator (SG), the underlying mechanism considering complete low-voltage ride-through (LVRT) processes of renewable energy equipment remains to be studied. Taking the SG and doubly fed induction generator (SG-DFIG) parallel system as an objective, this work divides its transient processes into four different stages: pre-fault (stage 1), during-fault (stage 2), early post-fault (stage 3), and late post-fault (stage 4), based on the LVRT of the DFIG, and establishes a transient model to describe the complete 4-stage LVRT processes. By studying the condition for entering the LVRT, it is found that vast majority of faults can cause the DFIG to enter the LVRT and make …

Class path: D3 — Physical Sciences › F21 — Energy

**Work 2.** Designability of RNA Targets with Up to Two Length-2 Helices

> RNA inverse folding asks for an RNA sequence whose prescribed secondary structure is the unique maximum-base-pair compatible fold. In the four-letter Watson-Crick model (A-U and C-G pairs only, no pseudoknots, and zero minimum base-pair span), Hales et al. introduced a separated-coloring certificate and an even-odd device, while Boury et al. generalized this to modulo-$m$ separability, gave an $O(n 2^m)$ decision algorithm, and guaranteed designability when every helix has length at least 3. We prove that the guarantee still holds when a motif-free target has at most two maximal helices of length 2, no maximal helix of length 1, and all remaining helices of length at least 3. The proof builds on Boury et al.'s local helix-coloring transfers and adds a global counting argument showing that the demands created by at most two short helices can always be coordinated. This is a structural …

Class path: D3 — Physical Sciences › F21 — Energy

## 3. uid `e5b71580eb`

Scheme: Free Decimal Correspondence

**Work 1.** Machian Gravity: Modeling Rotation Curves and Radial Acceleration in the SPARC Galaxy Sample

> This paper investigates the potential of Machian Gravity (MG), a five-dimensional theory of gravity, to explain the acceleration law governing rotationally bound systems, in particular spiral galaxies. MG was proposed as a framework capable of accounting for a range of astrophysical and cosmological phenomena -- including galactic rotation curves, mass distributions in galaxy clusters, and cosmic expansion -- without invoking additional dark components. In this study, we apply the MG acceleration law to a large sample of galaxies drawn from the SPARC database. Through a detailed analysis, we determine the optimal MG parameters for each individual galaxy, successfully fitting their observed rotation curves. Similar to Modified Newtonian Dynamics (MOND), our results indicate the existence of a characteristic acceleration scale associated with galactic dynamics, which regulates rotational …

Class path: 300 — Social sciences › 340 — Law

**Work 2.** Observation of Resonance of Kagome Flat Band Doublet

> The interplay between local and itinerant electrons underpins many correlated and topological quantum states. Kagome lattices provide an ideal platform by hosting both flat (localized states) and dispersive bands (itinerant states), yet direct spectroscopic evidence of their dynamical coupling has remained elusive. Here we report the long-sought flat band resonance in the quasi-two-dimensional kagome bilayer material CsCr6Sb6. Using angle-resolved photoemission spectroscopy, transport measurements, and combined density functional theory and dynamical mean-field theory, we identify coexisting flat band doublets and dispersive bands near the Fermi energy. Upon cooling, the flat and dispersive bands exhibit a pronounced enhancement of spectral weight and hybridization, directly evidencing flat band resonance. Crucially, this emergence coincides with the onset of short-range …

Class path: 500 — Science; Natural history › 530 — Physics › 538 — Magnetism

## 4. uid `c2dd6cfbe3`

Scheme: arXiv category taxonomy

**Work 1.** MIMIC: A Generative Multimodal Foundation Model for Biomolecules

> Biological function emerges from coupled constraints across sequence, structure, regulation, evolution, and cellular context, yet most foundation models in biology are trained within one modality or for a fixed forward task. We present MIMIC, a generative multimodal foundation model trained on our newly curated and aligned dataset, LORE, linking nucleic acid, protein, evolutionary, structural, regulatory, and semantic/contextual modalities within partially observed biomolecular states. MIMIC uses a split-track encoder-decoder architecture to condition on arbitrary subsets of observed modalities and reconstruct or generate missing components of molecular state across the genome, transcriptome, and proteome. Multimodal conditioning consistently improves MIMIC's sequence reconstruction relative to sequence-only inputs, while its learned representations enable state-of-the-art performance …

Class path: grp_cs — Computer Science › cs — Computer Science › cs.AI — Artificial Intelligence

**Work 2.** Using Symbolic Regression to Emulate the Radial Fourier Transform of the S\'ersic profile for Fast, Accurate and Differentiable Galaxy Profile Fitting

> Galaxy profile fitting is a ubiquitous technique that provides the backbone for photometric and morphological measurements in modern extragalactic surveys. A recent innovation in profile fitting algorithms is to render, or create, the model profile in Fourier space, which aims to provide faster and more accurate results. However, the most common parameterization, the S\'ersic profile, has no closed form Fourier transform, requiring the use of computationally expensive approximations. In this paper our goal is to develop an emulator to mimic the radial Fourier transform of the S\'ersic profile, for use in profile fitting. We first numerically compute the radial Fourier transform and demonstrate that it varies smoothly as a function of the S\'ersic index and $k$, the spatial frequency coordinate. Using this set of numerically calculated transforms as a training set, we use symbolic …

Class path: grp_physics — Physics › astro-ph — Astrophysics › astro-ph.IM — Instrumentation and Methods for Astrophysics

## 5. uid `d665ca03a3`

Scheme: arXiv category taxonomy

**Work 1.** Collaboration versus Specialization in Service Systems with Impatient Customers

> We study tandem queueing systems in which servers work more efficiently in teams than on their own and customers are impatient in that they may leave the system while waiting for service. Our goal is to determine the server assignment policy that maximizes the long-run average throughput. We show that when each server is equally skilled at all tasks, the optimal policy has all the servers working together at all times. We also provide a complete characterization of the optimal policy for Markovian systems with two stations and two servers when each server's efficiency may be task dependent. We show that the throughput is maximized under the policy which assigns one server to each station (based on their relative skill at that station) unless station 2 has no work (in which case both servers work at station 1) or the number of customers in the buffer reaches a threshold whose value we …

Class path: grp_math — Mathematics › math — Mathematics › math.PR — Probability

**Work 2.** Well-posedness of the obstacle problem for generalized Dean-Kawasaki equation

> We investigate the obstacle problem for generalized Dean--Kawasaki equations driven by correlated conservative noise, establishing the existence, uniqueness, and $L^1$-stability of stochastic kinetic solutions. Our core strategy combines a kinetic characterization of the Skorokhod condition with a precise description of the reflection measure term associated with the obstacle, in which the barrier substitutes the solution. This formulation makes the reflection mechanism explicit at the kinetic level and yields a stable framework adapted to $L^1$ doubling of variables method. Consequently, under a merely continuous obstacle and the same structural assumptions as in the obstacle-free setting, we obtain well-posedness over the full porous-medium regime, covering degenerate diffusion and the critical square-root noise coefficient. This extends the existing theory of obstacle problems for …

Class path: grp_math — Mathematics › math — Mathematics › math.PR — Probability

## 6. uid `79c0f7c8ec`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Transfinite Schreier families and cardinal invariants

> We study the dependence of the transfinite Schreier hierarchy on the choice of fundamental sequences for the countable limit ordinals. With each Schreier family we associate an interval endpoint function. We prove that the bounding number is equal to $\omega_1$ exactly when the fundamental sequences may be chosen so that these endpoint functions form an unbounded family in the eventual domination order. Equivalently, the hierarchy then satisfies the tail-covering property isolated by Shiliaev, or every infinite compact interval family has infinite intersection with some member of the hierarchy. We also prove that the dominating number is equal to $\omega_1$ exactly when the fundamental sequences may be chosen so that every compact family of finite subsets of $\{2,3,\ldots\}$ is contained in one Schreier family. The latter equivalence combines Fremlin's cofinality theorem for compact …

Class path: P — Language and literature › P1-1091 — Philology. Linguistics › P101-410 — Language. Linguistic theory. Comparative grammar › P121-149 — Science of language (Linguistics) › P128.A-128.Z — Special aspects, A-Z › P128.H53 — Hierarchy

**Work 2.** Beyond Solver Verdicts: Generative Reward Models for Autoformalization

> Neurosymbolic systems rely on mathematical solvers to guarantee reasoning correctness, yet solvers are fundamentally blind to whether a formal translation maintains strict reference-equivalence to a designated formalization. We formalize this vulnerability as Verdict-Preserving-Unfaithfulness (VPU): a failure mode where an incorrect encoding executes successfully and matches the expected verdict. We theoretically prove that structural, verdict-only verification heuristics are mathematically bounded to chance-level detection on these deceptively valid traces. To resolve this, we introduce Generative Verification (GenV), which distills an offline Z3-equivalence oracle into a reference-free, continuous reference-equivalence score by repurposing the language model's native vocabulary space. Mechanistic analysis via decision-projected logit lenses and sparse autoencoders shows this …

Class path: P — Language and literature › P1-1091 — Philology. Linguistics › P101-410 — Language. Linguistic theory. Comparative grammar › P121-149 — Science of language (Linguistics) › P128.A-128.Z — Special aspects, A-Z › P128.E65 — Equivalence

## 7. uid `9d7f079d27`

Scheme: Free Decimal Correspondence

**Work 1.** DrawVideo: Grounded and Faithful Multi-Shot Video Generation from Storyboard Keyframe Sketches

> Long video generation requires high-fidelity visual synthesis, coherent narrative organization, shot-level structure, and explicit user control. Existing text-to-video methods typically generate videos from a single long-form prompt, making it difficult for creators to directly control character pose, camera composition, spatial layout, and local motion. We propose DrawVideo, a sketch-guided and storyboard-driven framework that grounds multi-shot video generation in creator-provided spatial structure, appearance, and motion. Each shot is specified by a black-and-white sketch, an appearance prompt, and a motion prompt. DrawVideo first generates a structure-aligned reference keyframe, expands the motion description into derivative keyframes representing ordered action states, and then synthesizes local video clips between adjacent keyframes. We further introduce SketchLongVideo, to the …

Class path: 200 — Religion › 210 — Natural theology; Secularism › 213 — Creation; Evolution -- Religious aspects

**Work 2.** Light-induced Floquet spin-triplet Cooper pairs in unconventional magnets

> The recently predicted unconventional magnets offer a new ground for exploring the formation of nontrivial spin states due to their inherent nonrelativistic momentum-dependent spin splitting. In this work, we consider unconventional magnets with $d$- and $p$-wave parities, and investigate the effect of time-periodic light drives for inducing the formation of spin-triplet phases in the normal and superconducting states. In particular, we consider unconventional magnets without and with conventional superconductivity under linearly and circularly polarized light drives and treat the time-dependent problem within Floquet formalism, which naturally unveils photon processes and Floquet bands determining the emergent phenomena. We demonstrate that the interplay between unconventional magnetism and light gives rise to a non-trivial light-matter coupling which governs the emergence of Floquet …

Class path: 500 — Science; Natural history › 530 — Physics › 535 — Optics; Light

## 8. uid `ff2e916430`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** An elliptic K3 surface X/Q(t) with Mordell-Weil rank 17, I: Formulas for X and base changes of rank 18 and 19

> In [Elkies 2006, Elkies 2007] we announced a K3 surface X/Q with N\'{e}ron--Severi group NS(X) = NS_Q(X) of rank 19 and an elliptic fibration of Mordell--Weil rank 17. This is the largest possible Mordell--Weil rank over Q(t) for an elliptic K3 surface. One of the fibers of this surface is the elliptic curve of rank at least 28 that was the elliptic curve of highest rank known during the years 2006--2024. We further announced that there are quadratic base changes of this fibration to elliptic surfaces of Mordell--Weil rank 18 over Q(t), and that there are pairs of such quadratic base changes whose compositum is an elliptic surface of Mordell--Weil rank 19 over an elliptic curve E_0/Q with #(E_0(Q)) = \infty, whence there are infinitely many elliptic curves of rank at least 19 over Q. We exhibit one such pair. A subsequent paper will show (using a technique also announced in [Elkies …

Class path: 14-XX — Algebraic geometry › 14Jxx — Surfaces and higher-dimensional varieties › 14J28 — \(K3\) surfaces and Enriques surfaces

**Work 2.** Infinitely many elliptic curves over $\mathbb{Q}(i)$ of exact ranks 4 and 6 with $j$-invariant 1728

> For each $r\in\{4,6\}$, we construct an explicit one-parameter family of elliptic curves over $\mathbb{Q}(i)$ containing infinitely many pairwise nonisomorphic curves genuinely defined over $\mathbb{Q}(i)$ with $j$-invariant $1728$ and rank exactly $r$. We construct explicit $\mathbb{Q}(i)$-rational points to bound the ranks from below. Kai's theorem on prime values of linear patterns over number fields provides specializations with controlled local behavior, allowing us to obtain matching upper bounds via $[1+i]$-descent. The construction extends the strategy of the author's earlier rank-$2$ paper by replacing a symmetric Gaussian-prime configuration with systems of binary linear forms satisfying several complementary square identities. In the rank-$6$ case, the support vectors attached to the three constructed points and $(0,0)$ span the self-dual Reed-Muller code $\mathrm{RM}(1,3)$, …

Class path: 11-XX — Number theory › 11Gxx — Arithmetic algebraic geometry (Diophantine geometry) › 11G05 — Elliptic curves over global fields

## 9. uid `2781f89493`

Scheme: arXiv category taxonomy

**Work 1.** Production of Jets before Neutron Star Mergers

> We demonstrate that magnetospheric interactions between merging neutron stars (NSs) generate dual-jetted current outflows, analogous to the Alfv\'{e}n wings observed during planetary interactions in the Solar System. Using 3D relativistic MHD simulations, we model the interaction as a conducting sphere moving through a highly magnetized plasma of the companion's magnetosphere. Unusually, the interaction operates in a regime that is relativistic yet sub-Alfv\'{e}nic. Electromagnetic draping amplifies magnetic fields in a narrow layer near the stellar surface, leading to the generation of electric currents along the local magnetic field. The generation of beamed outflows enhances the instantaneous power of the pulsar-like radio and high-energy emission, produces spin/orbital modulations, and is likely to lead to observable precursor emission preceding the main gravitational wave event.

Class path: grp_physics — Physics › astro-ph — Astrophysics › astro-ph.HE — High Energy Astrophysical Phenomena

**Work 2.** An Adaptive Proximal Framework for Stochastic Weakly Convex Optimization

> Many nonsmooth, nonconvex objectives in learning and signal recovery are $\rho$-weakly convex. We minimize such a function when the weak-convexity parameter $\rho$ is unknown and only stochastic candidate and function-difference information is available. We propose the Adaptive Prox-Guided Scheme (APS), a single-trial framework that adapts the proximal parameter online and bidirectionally through a noisy descent test. APS attains a high-probability $O(\varepsilon^{-2})$ iteration bound for Moreau-envelope stationarity. The result allows biased, heavy-tailed function-difference estimates, while the candidate oracle need only be sufficiently accurate with constant probability when the proximal parameter is small and may be arbitrary otherwise. APS does not need to identify which case applies. A model-based extension covers adaptive proximal-point, prox-linear, and proximal-gradient …

Class path: grp_math — Mathematics › math — Mathematics › math.OC — Optimization and Control

## 10. uid `0afedbbb43`

Scheme: Free Decimal Correspondence

**Work 1.** Data-driven discovery of roughness descriptors for surface characterization and intimate contact modeling of unidirectional composite tapes

> Unidirectional tapes surface roughness determines the evolution of the degree of intimate contact required for ensuring the thermoplastic molecular diffusion and the associated inter-tapes consolidation during manufacturing of composite structures. However, usual characterization of rough surfaces relies on statistical descriptors that even if they are able to represent the surface topology, they are not necessarily connected with the physics occurring at the interface during inter-tape consolidation. Thus, a key research question could be formulated as follows: Which roughness descriptors simultaneously enable tape classification-crucial for process control-and consolidation modeling via the inference of the evolution of the degree of intimate contact, itself governed by the process parameters?. For providing a valuable response, we propose a novel strategy based on the use of Rank …

Class path: 100 — Philosophy and psychology › 110 — Metaphysics › 113-119 — Cosmology › 116 — Movement (Philosophy); Evolution (in the philosophical sense)

**Work 2.** A Theoretical Study of the Stellar Parameters and Surface Nucleosynthesis of the Extremely Metal-Poor RGB Giant BD$-18^{\circ}$5550 - Part I: Effects of $\alpha$-enhancement and Internal Mixing

> We present a detailed stellar evolution study of BD$-18^{\circ}$5550, an extremely metal-poor red giant star in the Galactic halo. Such stars are among the oldest known objects and preserve chemical signatures of the earliest stages of Galactic history. BD$-18^{\circ}$5550 exhibits $\alpha$-enhancement, $\rm[\alpha/Fe]=+0.4$, typical of Population II halo stars, with enhanced abundances of oxygen, magnesium, and other $\alpha$-elements relative to iron. Using the \texttt{MESA} stellar evolution code, we perform the first direct determination of its mass, age, radius, and luminosity by simultaneously fitting eight observed stellar properties and multiple chemical abundance ratios. Our analysis yields a current mass of $\rm M=0.76,M_{\odot}$, radius of $R=38.4,R_{\odot}$, luminosity of $\log L=2.8,L_{\odot}$, and an age of $t=11.87$ Gyr, with a 7-fold improvement in age precision. The …

Class path: 100 — Philosophy and psychology › 110 — Metaphysics › 113-119 — Cosmology › 116 — Movement (Philosophy); Evolution (in the philosophical sense)

## 11. uid `b56dcd3771`

Scheme: arXiv category taxonomy

**Work 1.** A Many to One Discrete Auditory Transform

> A many to one discrete auditory transform is presented to map a sound signal to a perceptually meaningful spectrum on the scale of human auditory filter band widths (critical bands). A generalized inverse is constructed in closed analytical form, preserving the band energy and band signal to noise ratio of the input sound signal. The forward and inverse transforms can be implemented in real time. Experiments on speech and music segments show that the inversion gives a perceptually equivalent though mathematically different sound from the input.

Class path: grp_math — Mathematics › math — Mathematics › math.NA — Numerical Analysis

**Work 2.** Energy-Preserving and Passivity-Consistent Numerical Discretization of Port-Hamiltonian Systems

> In this paper we design discrete port-Hamiltonian systems systematically in two different ways, by applying discrete gradient methods and splitting methods respectively. The discrete port-Hamiltonian systems we get satisfy a discrete notion of passivity, which lets us, by choosing the input appropriately, make them globally asymptotically stable with respect to an equilibrium point. We test methods designed using the discrete gradient approach in numerical experiments, and the results are encouraging when compared to relevant existing integrators of identical order.

Class path: grp_math — Mathematics › math — Mathematics › math.NA — Numerical Analysis

## 12. uid `7e540c8d74`

Scheme: UNESCO Thesaurus

**Work 1.** Personality Expression Across Contexts: Linguistic and Behavioral Variation in LLM Agents

> Large Language Models (LLMs) can be conditioned with explicit personality prompts, yet their behavioral realization often varies depending on context. This study examines how identical personality prompts lead to distinct linguistic, behavioral, and emotional outcomes across four conversational settings: ice-breaking, negotiation, group decision, and empathy tasks. Results show that contextual cues systematically influence both personality expression and emotional tone, suggesting that the same traits are expressed differently depending on social and affective demands. This raises an important question for LLM-based dialogue agents: whether such variations reflect inconsistency or context-sensitive adaptation akin to human behavior. Viewed through the lens of Whole Trait Theory, these findings highlight that LLMs exhibit context-sensitive rather than fixed personality expression, …

Class path: 4 — Social and human sciences › 4.10 — Psychology › c391 — Personality

**Work 2.** From $\log 2$ to $\pi/2$: the sharp asymptotic inradius of polynomial lemniscates

> Let $R_n$ be the infimum of the inradii of $\{z:|p(z)|<1\}$ over monic degree-$n$ polynomials whose zeros lie in the closed unit disk. We prove $nR_n\to\pi/2$, matching the asymptotic obstruction supplied by $z^n-1$. We first establish the exact universal radius $2^{1/n}-1$ for disks centered at zeros, which recovers the $(\log2)/n$ bound. Small inradius then forces radial concentration of the zeros and decay of their low reciprocal moments. These estimates give an entire limit with a modulus reflection identity; a second rescaling produces an exponential tangent and strict sublevel disks of every radius below $\pi/2$. The proof is accompanied by a Lean~4 formalization and a step-by-step source index.

Class path: 4 — Social and human sciences › 4.10 — Psychology › c391 — Personality › c12147 — Identity

## 13. uid `1ac9adf0fd`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** An Implicit, Moment-Conserving Low-Rank Tensor Method for the Hybrid Kinetic-Ion Fluid-Electron Plasma Model

> Quasi-neutral hybrid models for simulating plasmas, with kinetic ions and fluid electrons, have shown great promise for approaching the multi-scale nature of plasmas. In this paper we present a novel hybrid scheme for electromagnetic settings that conserves mass, momentum, and energy, employing a low-rank tensor method utilizing the Tensor Train decomposition to avoid the \enquote{curse of dimensionality.} Electrons are simulated as a continuous fluid, while the kinetic ions are modeled as a distribution function governed by the Vlasov equation. The Vlasov equation is updated with the implicit midpoint method. We use a modified version of a conservative truncation method to guarantee conservation of mass, momentum, and energy throughout the method, which corrects the truncated distribution function without needing to advance a separate set of fluid-moment equations. A series of …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.01 — Applied mathematics, analysis and probability › 02.01.01.02 — Matrix, tensor and polynomial computation

**Work 2.** Singly resonant InGaP microresonators for efficient, low-threshold second-order nonlinear optics

> Strong $\chi^{(2)}$ interactions at low optical powers are key to scalable nonlinear and quantum photonics. Although doubly resonant microresonators exhibit exceptional efficiencies, they require simultaneous resonance and phase matching of widely separated optical frequencies, making them inherently sensitive to fabrication and operating conditions. Singly resonant cavities eliminate this constraint by resonating at only the fundamental frequency while the second-harmonic field propagates in a single pass, but they have traditionally sacrificed nonlinear efficiency. Here, we demonstrate a singly resonant InGaP-on-insulator microresonator that overcomes this tradeoff. Combining the large $\chi^{(2)}$ nonlinearity of InGaP with high-quality-factor resonators, we achieve efficient and widely tunable second-harmonic generation, broadband quantum-frequency-comb generation, and low-threshold …

Class path: 02 — Mathematics and physical sciences › 02.02 — Physics › 02.02.02 — Condensed matter, materials and chemistry › 02.02.02.04 — Photonics and nonlinear optics

## 14. uid `e169cef6ce`

Scheme: arXiv category taxonomy

**Work 1.** Tight Generalization Bounds for Noiseless Inverse Optimization

> Inverse optimization (IO) seeks to infer the parameters of a decision-maker's objective from observed context--action data. We study noiseless IO, where demonstrations are generated by a ground-truth objective. We provide a high-probability ${O}(\frac{d}{T})$ generalization bound for the induced action set, where $d$ is the number of unknown parameters and $T$ is the size of the training dataset. We strengthen these guarantees under additional conditions that ensure uniqueness of the chosen action, bringing our IO guarantees in line with best-arm identification results in the bandit literature. We further show that the ${O}(\frac{d}{T})$ rate is tight over all consistent estimators considered here, and extend the result to both instantaneous and cumulative regret. Notably, the resulting regret lower bound matches the corresponding upper bounds in the adversarial setting, indicating that …

Class path: grp_stat — Statistics › stat — Statistics › stat.ML — Machine Learning

**Work 2.** Parameter-Free Dynamic Regret for Unconstrained Linear Bandits

> We study dynamic regret minimization in unconstrained adversarial linear bandit problems. In this setting, a learner must minimize the cumulative loss relative to an arbitrary sequence of comparators $\boldsymbol{u}_1,\ldots,\boldsymbol{u}_T$ in $\mathbb{R}^d$, but receives only point-evaluation feedback on each round. We provide a simple approach to combining the guarantees of several bandit algorithms, allowing us to optimally adapt to the number of switches $S_T = \sum_t\mathbb{I}\{\boldsymbol{u}_t \neq \boldsymbol{u}_{t-1}\}$ of an arbitrary comparator sequence. In particular, we provide the first algorithm for linear bandits achieving the optimal regret guarantee of order $\mathcal{O}\big(\sqrt{d(1+S_T) T}\big)$ up to poly-logarithmic terms without prior knowledge of $S_T$, thus resolving a long-standing open problem.

Class path: grp_cs — Computer Science › cs — Computer Science › cs.LG — Machine Learning

## 15. uid `5b4fda2df2`

Scheme: UNESCO Thesaurus

**Work 1.** Integer Factorization via Tensor Network Schnorr's Sieving

> Classical public-key cryptography standards rely on the Rivest-Shamir-Adleman (RSA) encryption protocol. The security of this protocol is based on the exponential computational complexity of the most efficient classical algorithms for factoring large semiprime numbers into their two prime components. Here, we address RSA factorization building on Schnorr's mathematical framework where factorization translates into a combinatorial optimization problem. We solve the optimization task via tensor network methods, a quantum-inspired classical numerical technique. This tensor network Schnorr's sieving algorithm displays numerical evidence of polynomial scaling of resources with the bit-length of the semiprime. We factorize RSA numbers up to 100 bits and assess how computational resources scale through numerical simulations up to 130 bits, encoding the optimization problem in quantum systems …

Class path: 6 — Politics, law and economics › 6.75 — Organization and management › c672 — Management techniques › c2567 — Operations research › c6659 — Optimization

**Work 2.** GPU Acceleration of Learning With Errors KEMs Using OpenACC for Post-Quantum Cryptography

> Shor's algorithm proved that asymmetric cryptographic protocols based on the integer factorization and discrete logarithm problems are no longer safe in a world with large-scale quantum computers. As a result, Post-Quantum Cryptography (PQC) has been developed over the last few years, seeking cryptographic primitives resistant to quantum attacks. One of the main hard problems underlying PQC schemes is the Learning with Errors (LWE) problem, which is significantly more computationally intensive than its classical predecessors. In this work, we present a Key Encapsulation Mechanism (KEM) based on plain LWE and develop a GPU-oriented implementation using OpenACC. We evaluate the performance of our accelerated application in terms of both time-to-solution and energy-to-solution, considering bare-metal and containerized executions across multiple NVIDIA GPU models and generations. Our …

Class path: 1 — Education › 1.05 — Educational sciences and environment › c5 — Learning

## 16. uid `3cbf51f894`

Scheme: Free Decimal Correspondence

**Work 1.** Emergence of Resonating Valence-Bond Correlations in Stretched Graphene

> Electronic correlations in graphene are generally considered weak due to the large bandwidth of its $\pi$ electrons. Here we show that tensile expansion of the honeycomb lattice provides a direct route to enhancing correlation effects. Using variational and diffusion quantum Monte Carlo, we compare a conventional Jastrow-Slater determinant wave function with a resonating-valence-bond (RVB) Jastrow-antisymmetrized geminal product ansatz for a series of stretched graphene lattices. We find that the energy gain of the RVB state relative to the single-determinant description increases with bond expansion up to a critical strain $\delta_{\mathrm{cr}}$, and decreases beyond it, revealing a nonmonotonic evolution of electronic correlations. The crossover is found to occur in the range $15\% < \delta_{\mathrm{cr}} < 20\%$, in agreement with mechanical stability limits. This behavior indicates a …

Class path: 100 — Philosophy and psychology › 110 — Metaphysics › 113-119 — Cosmology › 116 — Movement (Philosophy); Evolution (in the philosophical sense)

**Work 2.** Nonlinear Hall responses in tunable nodal Dirac semimetals

> We investigate the nonlinear Hall responses in tunable two-dimensional Dirac materials. In particular, we study quantum geometry-driven second and third-order nonlinear responses in a time-reversal symmetric Dirac semimetal that can host single-node, double-node, and nodal-ring depending on the model parameters. We find that the second-order Hall (SOH) response, which originates from the Berry curvature dipole, is enhanced in the single-node semimetallic phase as compared to the double-node case when inversion symmetry is broken. In contrast, the SOH response vanishes in the nodal-ring semimetal as the inversion symmetry is retained. Notably, the third-order Hall response due to Berry connection polarizability becomes much larger in the nodal-ring Dirac semimetal, especially when the Fermi energy lies near the band edge, than in the single- and double-node Dirac semimetals. The reason …

Class path: 500 — Science; Natural history › 510 — Mathematics › 513 — Geometry

## 17. uid `9963ed7b8e`

Scheme: OpenAlex Topics

**Work 1.** Deep Hilbert--Galerkin Methods for Infinite-Dimensional PDEs and Optimal Control

> We develop deep learning-based approximation methods for fully nonlinear second-order PDEs on separable Hilbert spaces, such as HJB equations for infinite-dimensional control, by parameterizing solutions via Hilbert--Galerkin Neural Operators (HGNOs). We prove the first Universal Approximation Theorems (UATs) which are sufficiently powerful to address these problems, based on novel topologies for Hessian terms and corresponding novel continuity assumptions on the fully nonlinear operator. These topologies are non-sequential and non-metrizable, making the problem delicate. In particular, we prove UATs for functions on Hilbert spaces, together with their Fr\'echet derivatives up to second order, and for unbounded operators applied to the first derivative, ensuring that HGNOs are able to approximate all the PDE terms. For control problems, we further prove UATs for optimal feedback …

Class path: D3 — Physical Sciences › F31 — Physics and Astronomy › S3109 — Statistical and Nonlinear Physics › T11206 — Model Reduction and Neural Networks

**Work 2.** Sample-efficient LLM Optimization with Reset Replay

> Recent advancements in LLM post-training, particularly through reinforcement learning and preference optimization, are key to boosting their reasoning capabilities. However, these methods often suffer from low sample efficiency and a susceptibility to primacy bias, a phenomenon where overfitting to initial experiences diminishes network plasticity and damages the learning process. To address these challenges, we introduce LLM optimization with Reset Replay (LoRR), a general and powerful plugin for enhancing sample efficiency in preference-based optimization. Its core mechanism enables high-replay training to maximize the utility of each data batch. To mitigate overfitting, LoRR orchestrates a periodic reset strategy that reuses the initial data and policy to maintain network plasticity, and further adopts a hybrid optimization objective to better exploit training data. Extensive …

Class path: D3 — Physical Sciences › F17 — Computer Science › S1705 — Computer Networks and Communications

## 18. uid `0b81fa5e1c`

Scheme: Free Decimal Correspondence

**Work 1.** Flow3r: Factored Flow Prediction for Scalable Visual Geometry Learning

> Current feed-forward 3D/4D reconstruction systems rely on dense geometry and pose supervision -- expensive to obtain at scale and particularly scarce for dynamic real-world scenes. We present Flow3r, a framework that augments visual geometry learning with dense 2D correspondences (`flow') as supervision, enabling scalable training from unlabeled monocular videos. Our key insight is that the flow prediction module should be factored: predicting flow between two images using geometry latents from one and pose latents from the other. This factorization directly guides the learning of both scene geometry and camera motion, and naturally extends to dynamic scenes. In controlled experiments, we show that factored flow prediction outperforms alternative designs and that performance scales consistently with unlabeled data. Integrating factored flow into existing visual geometry architectures …

Class path: 500 — Science; Natural history › 510 — Mathematics › 513 — Geometry

**Work 2.** THYME: Temporal Hierarchical-Cyclic Interactivity Modeling for Video Scene Graphs in Aerial Footage

> The rapid proliferation of video in applications such as autonomous driving, surveillance, and sports analytics necessitates robust methods for dynamic scene understanding. Despite advances in static scene graph generation and early attempts at video scene graph generation, previous methods often suffer from fragmented representations, failing to capture fine-grained spatial details and long-range temporal dependencies simultaneously. To address these limitations, we introduce the Temporal Hierarchical Cyclic Scene Graph (THYME) approach, which synergistically integrates hierarchical feature aggregation with cyclic temporal refinement to address these limitations. In particular, THYME effectively models multi-scale spatial context and enforces temporal consistency across frames, yielding more accurate and coherent scene graphs. In addition, we present AeroEye-v1.0, a novel aerial video …

Class path: 700 — Arts › 750 — Painting; Art

## 19. uid `df03062fdc`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** DiffuSpeech: Silent Thought, Spoken Answer via Unified Speech-Text Diffusion

> Current speech language models generate responses directly without explicit reasoning, leading to errors that cannot be corrected once audio is produced. We introduce \textbf{``Silent Thought, Spoken Answer''} -- a paradigm where speech LLMs generate internal text reasoning alongside spoken responses, with thinking traces informing speech quality. To realize this, we present \method{}, the first diffusion-based speech-text language model supporting both understanding and generation, unifying discrete text and tokenized speech under a single masked diffusion framework. Unlike autoregressive approaches, \method{} jointly generates reasoning traces and speech tokens through iterative denoising, with modality-specific masking schedules. We also construct \dataset{}, the first speech QA dataset with paired text reasoning traces, containing 26K samples totaling 319 hours. Experiments show …

Class path: 01 — Computing, AI and engineering

**Work 2.** SpeechLLM-as-Judges: Towards General and Interpretable Speech Quality Evaluation

> Generative speech technologies are progressing rapidly, but evaluating the perceptual quality of synthetic speech remains a core challenge. Existing methods typically rely on scalar scores or binary decisions, which lack interpretability and generalization across tasks and languages. We present SpeechLLM-as-Judges, a new paradigm for enabling large language models (LLMs) to conduct structured and explanation-based speech quality evaluation. To support this direction, we introduce SpeechEval, a large-scale dataset containing 32,207 multilingual speech clips and 128,754 annotations spanning four tasks: quality assessment, pairwise comparison, improvement suggestion, and deepfake detection. Based on this resource, we develop SQ-LLM, a speech-quality-aware LLM trained with chain-of-thought reasoning and reward optimization to improve capability. Experimental results show that SQ-LLM …

Class path: 01 — Computing, AI and engineering › 01.01 — Language models, security and computing systems

## 20. uid `932878031e`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** The P behind Q: Empirical Evidence from Physical Drift in Put-Call Parity

> Put-call parity is a terminal-payoff identity, but its enforcement is capital-using. I study the carry gap, the annualized wedge between option-implied and OIS discount factors, in SPX and RUT index options. Quoted parity is tightly compressed, while the synthetic-traded forward channel leaves a systematic wedge. I interpret this wedge as an implementation premium under finite arbitrage capital. A drift-preserving GBM term, r {\mu}-hat {\tau}, improves in-sample and leave-one-year-out fit, especially in SPX. The evidence suggests that physical drift enters not option payoffs, but the process enforcing risk-neutral parity.

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.01 — Applied mathematics, analysis and probability › 02.01.01.03 — Finite element and numerical methods for PDEs

**Work 2.** A Metric Between Probability Distributions on Finite Sets of Different Cardinalities and Applications to Order Reduction

> With increasing use of digital control it is natural to view control inputs and outputs as stochastic processes assuming values over finite alphabets rather than in a Euclidean space. As control over networks becomes increasingly common, data compression by reducing the size of the input and output alphabets without losing the fidelity of representation becomes relevant. This requires us to define a notion of distance between two stochastic processes assuming values in distinct sets, possibly of different cardinalities. If the two processes are i.i.d., then the problem becomes one of defining a metric between two probability distributions over distinct finite sets of possibly different cardinalities. This is the problem addressed in the present paper. A metric is defined in terms of a joint distribution on the product of the two sets, which has the two given distributions as its …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.01 — Applied mathematics, analysis and probability

## 21. uid `12be1b332d`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Form Over Content In Gradient-Based Data Attribution Methods

> Data attribution methods using gradient similarity are widely used to analyze and select training data for large language models, but what gradient similarity actually measures is debated. Some interpret it as identifying task-relevant skills, while other work reports that surface form is the main factor. We resolve this debate for supervised fine-tuning examples by varying task and answer format independently. Specifically, we render benchmarks in different answer formats, such that datasets can share a task without a format or a format without a task. We find that gradient alignment follows the answer format, as benchmark pairs sharing an answer format align strongly (disattenuated cosine near 0.4), while same benchmarks rendered with different answer format classes show no alignment (near 0.0). We demonstrate that this ordering holds from the earliest pretraining checkpoints through …

Class path: B — Philosophy. Psychology. Religion › BQ — Buddhism › BQ7001-9800 — Modifications, schools, etc. › BQ7530-7950.2 — Tibetan Buddhism (Lamaism) › BQ7566 — Methods

**Work 2.** How Data Shapes RoPE Frequency Usage: From Positional Scale Matching to Length Generalization

> Rotary Position Embeddings (RoPE) provide transformers with a fixed grid of positional frequencies, yet trained models use these frequencies highly non-uniformly. We study what determines this frequency usage and propose a data-centered explanation: RoPE frequencies are selected to match the relative-distance structure of the training data. Viewing each frequency as a positional lens, we formalize a field-resolution tradeoff and show that, for a data-induced dependency profile of width $W$, the optimal frequency scales as $1/W$. This frequency-matching principle explains controlled observations on synthetic and text-based data, and suggests that the mid-low frequency bands observed in language models arise from the multi-scale dependency structure of natural language. We further connect frequency selection to position-interpolation-based length generalization: scaling frequencies down …

Class path: P — Language and literature › P1-1091 — Philology. Linguistics › P101-410 — Language. Linguistic theory. Comparative grammar › P121-149 — Science of language (Linguistics) › P128.A-128.Z — Special aspects, A-Z › P128.F73 — Frequency

## 22. uid `930b1e2b53`

Scheme: Free Decimal Correspondence

**Work 1.** A Comparison of some Weighted Numerical Radii of Hilbert Space Operators

> In recent years, several generalizations of the numerical radius for bounded linear operators on Hilbert spaces have been introduced and extensively studied. These generalizations provide refined tools for investigating operator inequalities and spectral properties. In this paper, we investigate several generalized numerical radii and establish relationships among them. We also establish several relationships among these generalized numerical radii and show that the $(s,t)$-weighted numerical radius can be represented as a rescaled form of the $t$-weighted numerical radius.

Class path: 600 — Industrial arts; Technology; Engineering › 670 — Manufactures › 676 — Paper; Paper products

**Work 2.** On the notion of persistence of excitation for linear switched systems

> The paper formulates the concept of persistence of excitation for discrete-time linear switched systems, and provides sufficient conditions for an input signal to be persistently exciting. Persistence of excitation is formulated as a property of the input signal, and it is not tied to any specific identification algorithm. The results of the paper rely on realization theory and on the notion of Markov-parameters for linear switched systems.

Class path: 600 — Industrial arts; Technology; Engineering › 670 — Manufactures › 676 — Paper; Paper products

## 23. uid `b534d515e6`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Adaptive Krylov Methods for Low-Rank Exponential Integrators

> Differential equations arise in numerous applications, particularly within scientific and technical contexts. Systems of stiff, time-dependent ordinary differential equations constitute the focus of this work. Exponential integrators are designed to solve such equations by integrating the linear part exactly, while simultaneously approximating the nonlinear part through a linear combination of $\varphi$-functions. By utilizing an augmented stiffness matrix, state-of-the-art methods like KIOPS and RK2EXPINT solve the linear part and evaluate linear combinations of $\varphi$-functions for the nonlinear part in a single step, effectively reducing the computational effort to a single matrix exponential evaluation. However, these classical approaches assume that the system is represented by matrices and vectors, potentially not utilizing the underlying high-dimensional structure. Tensors …

Class path: 15-XX — Linear and multilinear algebra; matrix theory › 15Axx — Basic linear algebra › 15A69 — Multilinear algebra, tensor calculus

**Work 2.** Pre-asymptotic Error Analysis of CIP-FEM and FEM for Helmholtz Equation with High Wave Number. Part II: $hp$ version

> In this paper, which is part II in a series of two, the pre-asymptotic error analysis of the continuous interior penalty finite element method (CIP-FEM) and the FEM for the Helmholtz equation in two and three dimensions is continued. While part I contained results on the linear CIP-FEM and FEM, the present part deals with approximation spaces of order $p \ge 1$. By using a modified duality argument, pre-asymptotic error estimates are derived for both methods under the condition of $\frac{kh}{p}\le C_0\big(\frac{p}{k}\big)^{\frac{1}{p+1}}$, where $k$ is the wave number, $h$ is the mesh size, and $C_0$ is a constant independent of $k, h, p$, and the penalty parameters. It is shown that the pollution errors of both methods in $H^1$-norm are $O(k^{2p+1}h^{2p})$ if $p=O(1)$ and are $O\Big(\frac{k}{p^2}\big(\frac{kh}{\sigma p}\big)^{2p}\Big)$ if the exact solution $u\in H^2(\Om)$ which …

Class path: 65-XX — Numerical analysis › 65Nxx — Numerical methods for partial differential equations, boundary value problems › 65N12 — Stability and convergence of numerical methods for boundary value problems involving PDEs

## 24. uid `2704a4d1bd`

Scheme: OpenAlex Topics

**Work 1.** Learning and simulating bosonic systems via finite-energy locality

> Bosonic devices promise applications in simulation, sensing and quantum error correction, but infinite-dimensional local Hilbert spaces obstruct the locality tools that make qubit dynamics efficiently learnable and simulable. We establish a finite-energy locality principle for geometrically local bosonic open systems satisfying photon-number moment propagation. It compares unbounded generators with Galerkin cutoffs, transferring finite-dimensional Lieb--Robinson, product-formula and circuit techniques to bosons with explicit errors. For this moment-controlled class, we obtain, to our knowledge, the first model-independent weak Lieb--Robinson bounds beyond Bose--Hubbard-type dynamics, together with quantitative Trotter and simulation guarantees for polynomial bosonic GKSL generators. As a central application, coherent-state preparation and local heterodyne detection suffice to learn …

Class path: D3 — Physical Sciences › F21 — Energy

**Work 2.** Optical Modulation Due to Energy Exchange Between Photonic and Exciton Modes in the Intermediate Coupling Regime

> Actively tunable photonic devices are vital for next-generation optoelectronics requiring rapid switching and high bandwidth. Although organic optoelectronic devices have found wide application, their use as optical modulators has been limited by low absorption in the critical near-infrared (NIR) region, slow response time, and weak nonlinearities. To address these limitations, we developed a scheme based on intermediate exciton-photon coupling in a NIR absorbing squaraine-dye based photonic structure. Using energy-momentum resolved pump-probe spectroscopy, we show that the sign and magnitude of the optical response of our system depends strongly on the energy detuning between the excitonic and photonic modes. These data are analyzed using temporal coupled-mode theory to show that near resonance, a distinct energy exchange process emerges in the cross-over regime between strong and weak …

Class path: D3 — Physical Sciences › F21 — Energy

## 25. uid `6e7c68c954`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Broadband Chromatic Dispersion of Thermo-refractive Coefficients and its Impact in Silicon Nitride Nonlinear Photonics

> The thermo-refractive effect is a cornerstone of frequency and phase tuning in photonic integrated circuits. In particular, it enables control of phase-matching for integrated nonlinear processes. Chromatic dispersion of the group and effective refractive indices and modal confinement are standard considerations in design, but material thermo-refractive coefficients (TRCs) are typically taken to be fixed for the guiding and cladding materials. Here, we demonstrate that the assumption of non-dispersive TRCs across an octave of bandwidth between the telecom and visible results in a significant discrepancy between measured and simulated resonance frequencies of an integrated Si3N4/SiO2 microring resonator. We uncover a 7 % variation in Si3N4 and SiO2 material TRCs across this range, finding that the variation of dneff /dT from material TRCs is 1.3 times that from modal confinement. This …

Class path: Q — Science › QC — Physics › QC170-197 — Atomic physics. Constitution and properties of matter › QC174.24.A-174.24.Z — Special types of wave mechanics, A-Z › QC174.24.N6 — Nonlinear

**Work 2.** From Memorization to Absorption: Mixed-Policy RL for Continual Knowledge Injection

> Continual knowledge injection is essential for keeping large language models up-to-date in a fast-evolving world. Existing methods rely on supervised fine-tuning (SFT), which memorizes injected facts in their training format but fails to generalize across paraphrasing, document combinations, and reasoning. To address this, we propose Golden-GRPO Injection (GRIN), a three-stage self-learning framework for continual knowledge injection. Golden-GRPO is a mixed-policy reinforcement learning algorithm designed specifically for knowledge injection, which injects a golden answer to provide learning signal even when on-policy rollouts fail on novel facts. We further introduce Blank and Counter, two document-level benchmarks targeting novel acquisition and counterfactual overwrite respectively, each evaluating single-fact recall, multi-source retrieval, and inferential reasoning. Our experiments …

Class path: S — Agriculture › SF — Animal culture › SF600-1100 — Veterinary medicine › SF915-919.5 — Veterinary pharmacology › SF919.5.A-919.5.Z — Special, A-Z › SF919.5.I53 — Injection

## 26. uid `f8d5ec7e2a`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** GroundingAnomaly: Spatially-Grounded Diffusion for Few-Shot Anomaly Synthesis

> The performance of visual anomaly inspection in industrial quality control is often constrained by the scarcity of real anomalous samples. Consequently, anomaly synthesis techniques have been developed to enlarge training sets and enhance downstream inspection. However, existing methods either suffer from poor integration caused by inpainting or fail to provide accurate masks. To address these limitations, we propose GroundingAnomaly, a novel few-shot anomaly image generation framework. Our framework introduces a Spatial Conditioning Module that leverages per-pixel semantic maps to enable precise spatial control over the synthesized anomalies. Furthermore, a Gated Self-Attention Module is designed to inject conditioning tokens into a frozen U-Net via gated attention layers. This carefully preserves pretrained priors while ensuring stable few-shot adaptation. Extensive evaluations on the …

Class path: H — Social sciences › HF — Commerce › HF5001-6182 — Business › HF5546-5548.6 — Office management › HF5548.4.A-5548.4.Z — Special computers, systems, or programs, A-Z › HF5548.4.F7 — Framework

**Work 2.** Hermes Seal: Zero-Knowledge Assurance for Autonomous Vehicle Communications

> The application of zero-knowledge proofs (ZKPs) in autonomous systems is an emerging area of research, motivated by the growing need for regulatory compliance, transparent auditing, and trustworthy operation in decentralized environments. zk-SNARK is a powerful cryptographic tool that allows a party (the prover) to prove to another party (the verifier) that a statement about its own internal state is true, without revealing sensitive or proprietary data about that state. This paper proposes Hermes Seal: a zk-SNARK-based ZKP framework for enabling privacy-preserving, verifiable communication in vehicle-to-vehicle (V2V) and vehicle-to-infrastructure (V2I) networks. The framework allows autonomous systems to generate cryptographic proofs of perception and decision-related computations without revealing proprietary models, sensor data, or internal system states, thereby supporting …

Class path: H — Social sciences › HF — Commerce › HF5001-6182 — Business › HF5546-5548.6 — Office management › HF5548.4.A-5548.4.Z — Special computers, systems, or programs, A-Z › HF5548.4.F7 — Framework

## 27. uid `8b75a7b2ea`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** Beyond $K$-Theory: Geometry and Holomorphy in Hyperbolic Band Theory

> Topological $K$-theory supplies the decisive stable classification principle for gapped free-fermion phases in Euclidean crystals once symmetry and stabilization are fixed. In hyperbolic band theory, by contrast, it is less decisive for the full band problem: generalized momentum sectors vary through moduli spaces, but passing to ordinary $K$-classes collapses their geometric and holomorphic variation. We distinguish kinematical holomorphy, in which complex geometry organizes the sector spaces, from dynamical holomorphy, in which that geometry informs a Hamiltonian or projector, and formulate the resulting loss as an observable-factorization problem. For a compact hyperbolic surface $X$, we prove nonfactorization in three settings: fibrewise $K^0(X)$, occupied-state $K^0(B)$, and operator-algebraic $K$-theory. The affected quantities include spectra and Higgs spectral curves, the Berry …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.02 — Pure mathematics › 02.01.02.02 — Functional analysis and operator theory

**Work 2.** New recursive approximations for variable-order fractional operators with applications

> To broaden the range of applicability of variable-order fractional differential models, reliable numerical approaches are needed to solve the model equation. In this paper, we develop Laguerre spectral collocation methods for solving variable-order fractional initial value problems on the half line. Specifically, we derive three-term recurrence relations to efficiently calculate the variable-order fractional integrals and derivatives of the modified generalized Laguerre polynomials, which lead to the corresponding fractional differentiation matrices that will be used to construct the collocation methods. Comparison with other existing methods shows the superior accuracy of the proposed spectral collocation methods.

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics

## 28. uid `dd004fb9e4`

Scheme: arXiv category taxonomy

**Work 1.** Linear convergence of iterative contour integral-based eigensolvers for nonlinear eigenvalue problems

> Solving nonlinear eigenvalue problems is an important and challenging task in scientific computing. Contour integral-based approaches are attractive for such eigenvalue problems because they reliably target all eigenvalues in a prescribed domain. However, unlike in the linear case, many traditional methods of this type, such as Beyn's method, lack an inherent iterative refinement mechanism. Consequently, achieving high accuracy requires high-quality quadrature rules for approximating the contour integral, which often leads to prohibitive computational costs. A notable exception is the so-called NLFEAST algorithm, which combines contour integral techniques with a nonlinear Rayleigh--Ritz extraction step. In this work, we propose a general framework of iterative contour integral-based methods for nonlinear eigenvalue problems that includes NLFEAST. This allows us to prove linear …

Class path: grp_math — Mathematics › math — Mathematics › math.NA — Numerical Analysis

**Work 2.** Machine Learning the Strong Disorder Renormalization Group Method for Disordered Quantum Spin Chains

> We train machine learning algorithms to infer the entanglement structure of disordered long-range interacting quantum spin chains by learning from the strong disorder renormalisation group (SDRG) method. The system consists of $S=1/2$-quantum spins coupled by antiferromagnetic power-law interactions with decay exponent $\alpha$ at random positions on a one-dimensional chain. Using SDRG as a physics-informed teacher, we compare a Random Forest classifier as a classical baseline with a graph neural network (GNN) that operates directly on the interaction graph and learns a bond-ranking rule mirroring the SDRG decimation policy. The GNN achieves a disorder-averaged pairing accuracy close to one and reproduces the entanglement entropy $S(\ell)$ in excellent quantitative agreement with SDRG across all subsystem sizes and interaction exponents. RG flow heat maps confirm that the GNN learns the …

Class path: grp_physics — Physics › cond-mat — Condensed Matter › cond-mat.dis-nn — Disordered Systems and Neural Networks

## 29. uid `c02c0631bb`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** The role of anisotropy in $f(Q)$ gravity: insights from cosmological observations

> We investigate the cosmological dynamics of Bianchi-I spacetime in symmetric teleparallel $f(Q)$ gravity through a dynamical system approach to analyse observational constraints. By reformulating the modified field equations into an autonomous system, we analyse two representative $f(Q)$ models and constrain their parameters using Pantheon Plus, DES Y5, DESI DR2, and compressed CMB data. The observational analysis yields consistent constraints across all dataset combinations and tightly bounds the anisotropic contribution, indicating that deviations from isotropy remain small. Both models reproduce the standard matter-dominated evolution and the observed late-time accelerated expansion while exhibiting distinct dark-energy dynamics. Model I undergoes a smooth phantom-divide crossing and approaches a de Sitter phase in the asymptotic future, whereas Model II evolves from an early phantom …

Class path: e — atoms › ec — period 2 elements › ecr — fluorine › ecra — attributes of #ecra › ecran — F-

**Work 2.** Observer effect modulates classification in a quantum epistemic framework

> The observer effect in quantum physics states that observation inevitably influences the system being observed. This work introduces an epistemic framework that treats the observer as an integral part of sensory information processing within entangled quantum systems, leading to subjective and probabilistic observation and inference. We propose fuzzy instance classification by encoding sensory input to align with the observer's pre-existing beliefs in a feature-attribute-truth value hierarchical model as 'bells' of quantum oscillators whose states represent the degree of activation, associated with quantum probability. We demonstrate that within this framework, sensory data evolve via interaction with quantum-based observer states during the pre-decision phase, as described by the Lindblad master equation, and are then classified adaptively using positive operator-valued measures …

Class path: b — spacetime › bp — processes › bpe — states

## 30. uid `0155323c8a`

Scheme: arXiv category taxonomy

**Work 1.** Domain Adaptation with Target Information via Doubly-Anchored Distributionally Robust Optimization

> Domain Adaptation (DA) often lacks worst-case guarantees, while Distributionally Robust Optimization (DRO) based only on source data centers its ambiguity set at the source law and ignores available target structure. To bridge this gap, we introduce a doubly-anchored DRO framework whose ambiguity set is the intersection of $\phi$-divergence balls centered at the source law and a source-completed target reference law, the latter pairing the target covariate law with the source conditional law. We derive dual-induced adversarial bridge geometries for symmetric and asymmetric divergence pairings, notably introducing a Kullback--Leibler/squared-Hellinger (KL/HD) bridge. This asymmetric formulation yields a Lambert-$W$ geometry in which source-side exponential risk tilting and target-side Hellinger stabilization enter through distinct terms, attenuating, but not bounding, the effect of large …

Class path: grp_stat — Statistics › stat — Statistics › stat.ML — Machine Learning

**Work 2.** Causal Optimal Coupling for Gaussian Input-Output Distributional Data

> We study the problem of identifying an optimal coupling between input-output distributional data generated by a causal dynamical system. The coupling is required to satisfy prescribed marginal distributions and a causality constraint reflecting the temporal structure of the system. We formulate this problem as a Schr"odinger Bridge, which seeks the coupling closest - in Kullback-Leibler divergence - to a given prior while enforcing both marginal and causality constraints. For the case of Gaussian marginals and general time-dependent quadratic cost functions, we derive a fully tractable characterization of the Sinkhorn iterations that converges to the optimal solution. Beyond its theoretical contribution, the proposed framework provides a principled foundation for applying causal optimal transport methods to system identification from distributional data.

Class path: grp_eess — Electrical Engineering and Systems Science › eess — Electrical Engineering and Systems Science › eess.SY — Systems and Control

## 31. uid `76c3fdbd0f`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** A Method for Solving Cyclic Block Penta-diagonal Systems of Linear Equations

> A method for solving cyclic block three-diagonal systems of equations is generalized for solving a block cyclic penta-diagonal system of equations. Introducing a special form of two new variables the original system is split into three block pentagonal systems, which can be solved by the known methods. As such method belongs to class of direct methods without pivoting. Implementation of the algorithm is discussed in some details and the numerical examples are present.

Class path: G — Geography. Anthropology. Recreation › GV — Recreation. Leisure › GV557-1198.995 — Sports › GV861-1017 — Ball games: Baseball, football, golf, etc. › GV1015.5.A-1015.5.Z — Special topics, A-Z › GV1015.5.B56 — Block

**Work 2.** Concentrated sets and the Hurewicz property

> A set of reals $X$ is $\mathfrak{b}$-concentrated if it has cardinality at least $\mathfrak{b}$ and it contains a countable set $D\subseteq X$ such that each closed subset of $X$ disjoint with $D$ has size smaller than $\mathfrak{b}$. We present ZFC results about structures of $\mathfrak{b}$-concentrated sets with the Hurewicz covering property using semifilters. Then we show that assuming that the semifilter trichotomy holds, then each $\mathfrak{b}$-concentrated set is Hurewicz and even productively Hurewicz. We also show that the appearance of Hurewicz $\mathfrak{b}$-concentrated sets under the semifilter trichotomy is somewhat specific and the situation in the Laver model for the consitency of the Borel Conjecture is different.

Class path: G — Geography. Anthropology. Recreation › GV — Recreation. Leisure › GV557-1198.995 — Sports › GV861-1017 — Ball games: Baseball, football, golf, etc. › GV1015.5.A-1015.5.Z — Special topics, A-Z › GV1015.5.S48 — Set

## 32. uid `69277b65d1`

Scheme: arXiv category taxonomy

**Work 1.** Grid-cell firing fields lack local sixfold symmetry

> The firing fields of a grid cell form a hexagonal lattice, yet each field's intrinsic geometric structure remains unclear: Are individual grid fields radially symmetric or do they exhibit (weak) sixfold modulation inherited from the global lattice? To address this question, we quantified the within-field angular structure using harmonic analysis and per-cell matched simulations that preserved sampling statistics, field scale, and lattice geometry after correction for global elliptic deformation. We found that, across the investigated cells, the sixfold harmonic power fraction was consistent with values from matched locally circular reference fields. Experimental fields further showed substantially less sixfold modulation than matched simulations with an imposed local sixfold component, demonstrating sensitivity to sixfold structure at that level. This conclusion was supported by a …

Class path: grp_q-bio — Quantitative Biology › q-bio — Quantitative Biology › q-bio.NC — Neurons and Cognition

**Work 2.** ARISE-RL: Agentic Rubric-Grounded Iterative Self-Evolution with Reinforcement Learning

> Training open-ended agents via reinforcement learning (RL) is hindered by the lack of verifiable gold answers and scalable rubrics. Moreover, even near the model's capability boundary, long-horizon open-ended agentic tasks often yield brittle and unstable rewards, resulting in weak or noisy rollout contrast that obscures fine-grained optimization signals for group-based policy learning. To address these challenges, we propose ARISE-RL, a novel full-cycle self-evolution framework that couples a task/rubric Generator and a reasoning Solver through rubric-mediated co-evolution. The Generator grounds tool-related rubric criteria in real tool observations and is rewarded for producing valid, intermediate-difficulty tasks aligned with the Solver's evolving capability boundary. The Solver, in turn, learns from fine-grained rubric satisfaction signals through multi-step reasoning and tool use. …

Class path: grp_cs — Computer Science › cs — Computer Science › cs.AI — Artificial Intelligence

## 33. uid `6cafca18ba`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Weak Values Beyond the Weak Limit: Measurement-Strength Invariance Under Post-Selection

> Weak values are conventionally extracted in the limit of vanishing measurement strength. At finite strength, the corresponding post-selected conditional averages generally acquire nonlinear corrections and need not retain their weak value interpretation. Here we identify a class of measurement protocols for which this expectation fails: the conditional value reconstructed from the measuring probe is exactly independent of the probe strength and therefore coincides with the real part of the weak value. We first analyze this phenomenon in the measurement--disturbance protocol introduced by Lund and Wiseman. We then establish conditions under which this invariance persists, showing that it is determined by the interplay between the probe interaction, a subsequent measurement interaction with an ancillary apparatus, and the final post-selection. These results demonstrate that weak coupling, …

Class path: Q — Science › QC — Physics › QC770-798 — Nuclear and particle physics. Atomic energy. Radioactivity › QC794.8.A-794.8.Z — Special types of interactions, A-Z › QC794.8.W4 — Weak

**Work 2.** CMKL: Modality-Aware Continual Learning for Evolving Biomedical Knowledge Graphs

> Biomedical knowledge graphs are increasingly large, dynamic, and multimodal, driven by rapid advances in biotechnology such as high-throughput sequencing. Machine learning models can infer previously unobserved biomedical relationships and characterize biomedical entities in these graphs, but existing knowledge graph embedding methods and their continual learning extensions either assume static graph structure or fail to exploit multimodal information under evolving data distributions. They also apply uniform regularization across all model parameters, ignoring that different modalities may exhibit distinct forgetting dynamics as the graph evolves. We propose the Continual Multimodal Knowledge Graph Learner (CMKL), a CL framework for biomedical KGs that natively encodes structure, text, and molecules, fuses them through a Mixture-of-Experts (MoE) router, and protects previously learned …

Class path: Z — Bibliography. Library science. Information resources (General) › Z5051-7999 — Subject bibliography › Z6651-6655 — Mathematics (Table Z5 modified) › Z6654.A-6654.Z — Special topics, A-Z › Z6654.G8 — Graph

## 34. uid `ae3045192a`

Scheme: UNESCO Thesaurus

**Work 1.** Your LLM Agent Can Leak Your Data: Data Exfiltration via Backdoored Tool Use

> Tool-use large language model (LLM) agents are increasingly deployed to support sensitive workflows, relying on tool calls for retrieval, external API access, and session memory management. While prior research has examined various threats, the risk of systematic data exfiltration by backdoored agents remains underexplored. In this work, we present Back-Reveal, a data exfiltration attack that embeds semantic triggers into fine-tuned LLM agents. When triggered, the backdoored agent invokes memory-access tool calls to retrieve stored user context and exfiltrates it via disguised retrieval tool calls. We further demonstrate that multi-turn interaction amplifies the impact of data exfiltration, as attacker-controlled retrieval responses can subtly steer subsequent agent behavior and user interactions, enabling sustained and cumulative information leakage over time. Our experimental results …

Class path: 4 — Social and human sciences › 4.10 — Psychology › c387 — Cognition › c5556 — Memorization

**Work 2.** How to Detect Network Dependence in Latent Factor Models? A Bias-Corrected CD Test

> In a recent paper Juodis and Reese (2022) (JR) show that the application of the CD test proposed by Pesaran (2004) to residuals from panels with latent factors results in over-rejection. They propose a randomized test statistic to correct for over-rejection, and add a screening component to achieve power. This paper considers the same problem but from a different perspective, and shows that the standard CD test remains valid if the latent factors are weak in the sense the strength is less than half. In the case where latent factors are strong, we propose a bias-corrected version, CD*, which is shown to be asymptotically standard normal under the null of error cross-sectional independence and have power against network type alternatives. This result is shown to hold for pure latent factor models as well as for panel regression models with latent factors. The case where the errors are …

Class path: 2 — Science › 2.05 — Scientific approach › c106 — Scientific methods › c7121 — Simulation models

## 35. uid `255faa1afb`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** Rebuttals Move Peer-Review Scores, but Initial-Review Structure Bounds the Movement

> Author rebuttals are the main post-submission window in peer review, but their effect on reviewer scores remains hard to measure because score updates mix rebuttal content with initial score position, paper-level consensus, reviewer confidence, and discussion dynamics. We study ICLR 2024-2025 using 73,000 reviewer trajectories with externally archived pre- and post-rebuttal scores, and use LLMs only as measurement instruments. Gemini Flash 3.0 predicts implied pre-rebuttal scores from score-stripped review text. The resulting text-score offset predicts later movement, with score-increase rates rising from 8.3% when text reads below the assigned score to 31.9% when it reads above. Claude Opus 4.6 induces, and outcome-blinded Gemini Flash 3.0 validates, a 44-feature taxonomy of resolved reviewer-author exchanges, where 23 features replicate across model and held-out year under Bonferroni …

Class path: 01 — Computing, AI and engineering › 01.01 — Language models, security and computing systems › 01.01.01 — Language, speech and LLM agents › 01.01.01.04 — Language technology, society and bias

**Work 2.** TReVS: Integrating Textual Relevance and Visual Saliency for Efficient Vision-Language Model Token Pruning

> Vision-Language Models (VLMs) excel at visual understanding and reasoning but often incur substantial inference costs due to the large number of visual tokens. Recent visual token pruning methods increasingly follow a two-stage paradigm: they first remove visually redundant tokens after the vision encoder and then discard tokens irrelevant to the textual query within the Large Language Model (LLM). However, since the first stage typically relies solely on vision-encoder saliency, it may prematurely eliminate query-relevant tokens, depriving the subsequent text-guided stage of critical visual evidence. Our empirical analysis shows that incorporating query guidance into first-stage pruning better preserves task-relevant evidence and consistently improves performance over vision-only saliency-based pruning. We further find that high-variance attention heads are more sensitive to the …

Class path: 01 — Computing, AI and engineering

## 36. uid `85c9423438`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** A Single-Loop Bilevel Deep Learning Method for Optimal Control of Obstacle Problems

> Optimal control of obstacle problems arises in a wide range of applications and is computationally challenging due to its nonsmoothness, nonlinearity, and bilevel structure. Classical numerical approaches rely on mesh-based discretization and typically require solving a sequence of costly subproblems. In this work, we propose a single-loop bilevel deep learning method, which is mesh-free, scalable to high-dimensional and complex domains, and avoids repeated solution of discretized subproblems. The method employs constraint-embedding neural networks to approximate the state and control and preserves the bilevel structure. To train the neural networks efficiently, we propose a Single-Loop Stochastic First-Order Bilevel Algorithm (S2-FOBA), which eliminates nested optimization and does not rely on restrictive lower-level uniqueness assumptions. We analyze the convergence behavior of …

Class path: 68-XX — Computer science › 68Txx — Artificial intelligence › 68T07 — Artificial neural networks and deep learning

**Work 2.** Influence-Inspired Spectral Rotations for Extreme Low-Bit LLM Quantization

> We apply the influence-adaptive Walsh geometry of a companion theory paper (arXiv:2605.01637) to extreme low-bit weight-only LLM quantization. The recipe is one math-invariant transformation: WHT-rotate each linear layer's weight matrix and rescale its columns by per-coordinate Walsh-basis activation energy before handing off to a reconstruction-error quantizer (Intel auto-round). This biases per-group integer rounding toward high-spectral-energy channels. On four pretrained decoder-only models from 135M to 1.5B parameters, BBT-spectral reduces wikitext-2 perplexity by 15-58% relative to vanilla auto-round at W2A16; we also report a TinyLlama-1.1B auxiliary data point. Three extensions transfer the recipe to families it failed on: a per-head PCA matrix-Gamma replacement of q_norm/k_norm for Qwen3 attention (PPL 136.76 -> 88.99 on Qwen3-0.6B); an SO(2) per-pair rotation that commutes …

Class path: 68-XX — Computer science › 68Txx — Artificial intelligence › 68T07 — Artificial neural networks and deep learning

## 37. uid `504180e6d2`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** Low Precision Fortran -- Enabling Low Precision Floating Point Arithmetic in Modern Fortran

> Although Fortran is almost 70 years old, the language continues to evolve in order to keep pace with developments in computer science. In particular, a flexible type system was introduced that allows developers to specify the sizes of floating-point numbers and integers. In the latest revisions of the Fortran standard, portable type variants for IEEE 754 binary64 (double precision, real64) and binary32 (single precision, real32) were added. However, the rapid development of AI toolkits and accelerator hardware has created a strong focus on floating-point types of lower precision and lower memory usage than binary32. While the IEEE 754-2019 standard defines the binary16 type for representing half-precision numbers, the Fortran standard does not provide the real16 variant in the type system. In contrast, most C compilers support such a data type. In numerical linear algebra, there is …

Class path: a — forms › an — quantities › ann — units › annp — one

**Work 2.** TorchCraft: Unified binder design by inverting an all-atom structure predictor

> All-atom structure predictors model diverse molecular interactions, but using their learned structural priors for binder design remains challenging. Here we present TorchCraft, a unified binder-design framework that optimizes sequence logits through a frozen all-atom predictor. Implemented in TorchFold, TorchCraft combines confidence, contact, geometric, and sequence-prior objectives within a shared optimization procedure for minibinders, framework-conditioned VHHs, cyclic peptides, and ligand-binding proteins. Using pretrained AlphaFold 3 weights, TorchCraft generated representative minibinders and VHHs with experimentally measured binding across four targets in each format, without post hoc sequence redesign. Computational benchmarks further demonstrated the framework's applicability to cyclic peptides and ligand-conditioned pocket design. TorchCraft extends predictor inversion to …

Class path: f — molecules › ft — proteins › ftj — peptides

## 38. uid `65ffeb6d14`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** Token Efficient Task Execution via Application Behavior Modeling for Web Agents

> The strong performance of AI Agents across an impressive variety of tasks is driving an unprecedented investment in agentic infrastructures, however the cost of processing tokens is fast increasing. Web agents automate the execution of web-application tasks described in natural language, by analyzing the web-application's user interface (UI) and interacting with it. This work introduces OdoBot, a novel web-agent architecture that completes tasks at a fraction of the cost when compared to conventional web agents. This is achieved by leveraging a behavioral model of the underlying application constructed by analyzing successful task-execution demonstrations. Our experiments with 45 tasks on the Canvas Learning Management System (LMS) demonstrate that OdoBot uses 44% and 80% fewer tokens than two state-of-the-art competitor agents (Agent-E and WebVoyager), while also surpassing WebVoyager …

Class path: px63 — agents

**Work 2.** Easy-IIL: Reducing Human Operational Burden in Interactive Imitation Learning via Assistant Experts

> Interactive Imitation Learning (IIL) typically relies on extensive human involvement for both offline demonstration and online interaction. Prior work primarily focuses on reducing human effort in passive monitoring rather than active operation. Interestingly, structured model-based imitation approaches achieve comparable performance with significantly fewer demonstrations than end-to-end imitation learning policies in the low-data regime. However, these methods are typically surpassed by end-to-end policies as the data increases. Leveraging this insight, we propose Easy-IIL, a framework that utilizes off-the-shelf model-based imitation methods as an assistant expert to replace active human operation for the majority of data collection. The human expert only provides a single demonstration to initialize the assistant expert and intervenes in critical states where the task is approaching …

Class path: o — instincts › oa — attributes of #oa › oal — learning › oalt — imitation

## 39. uid `594c8b532f`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Topological obstructions to geometric positivity and negativity on Calabi-Yau manifolds

> We study whether the topology underlying a Calabi-Yau manifold can support natural geometric positivity or negativity structures. In even complex dimension $n \geq 4$ (assuming $b_2=1$ when $n \geq 6$), we strengthen a theorem of Oguiso-Peternell by proving that a Calabi-Yau manifold is not homeomorphic to a weak Fano $n$-fold. The same obstruction applies to K\"{a}hler manifolds with quasi-positive holomorphic sectional curvature. A transformation-group analogue excludes, in particular, symplectic manifolds admitting Hamiltonian circle actions with isolated fixed points. On the negative side, we show that the fundamental group of a Calabi-Yau manifold is not isomorphic to that of a K\"{a}hler hyperbolic manifold. Taken together, these results exhibit a common topological rigidity separating Calabi-Yau manifolds from several fundamental classes governed by geometric positivity or …

Class path: 53-XX — Differential geometry › 53Cxx — Global differential geometry › 53C55 — Global differential geometry of Hermitian and Kählerian manifolds

**Work 2.** On quasi-holomorphic homotopies of immersions of 3-manifolds into 5-manifolds

> The notion of a quasi-holomorphic homotopy of two immersions of a $3$-manifold into a $5$-manifold extends the notion of their regular homotopy by also allowing the homotopy to pass through instances of maps with an isolated singularity around which the path of the homotopy forms a cross-cap (complex Whitney umbrella). We describe the local form of such homotopies and explain connections with the theory of holomorphic map germs from $\mathbb{C}^2$ to $\mathbb{C}^3$. Our main result is a complete a description of how the fundamental group of the complement of the image of an immersion of a $3$-manifold into a $5$-manifold (i.e. its knot group) changes under a quasi-holomorphic homotopy. As a corollary we will see that certain quasi-holomorphic homotopies of the standard embedding of the $3$-sphere into the $5$-sphere do not change its knot group.

Class path: 57-XX — Manifolds and cell complexes › 57Rxx — Differential topology › 57R42 — Immersions in differential topology

## 40. uid `2bed8f174e`

Scheme: OpenAlex Topics

**Work 1.** SCOPE: Deterministic and Training-Free 3D UAV Deployment via Perimeter-based Heuristics

> Unmanned Aerial Vehicle (UAV) mounted Base Stations (UAV-BSs) provide flexible coverage for temporary hotspot scenarios; however, efficiently optimizing 3D deployment to satisfy heterogeneous user distributions remains a significant challenge. While Deep Reinforcement Learning (DRL) approaches have shown promise, they often suffer from prohibitive training overhead and poor generalization in cold-start scenarios where the user topology is unknown a priori. To address these limitations, this paper proposes Satisfaction-driven Coverage Optimization via Perimeter Extraction (SCOPE), which is a deterministic and training-free 3D deployment framework. Unlike existing heuristics that rely on fixed-altitude assumptions, SCOPE integrates a perimeter-based peeling strategy with the Welzl Smallest Enclosing Circle (SEC) algorithm to dynamically optimize 3D positions. Theoretically, we provide a …

Class path: D3 — Physical Sciences › F17 — Computer Science › S1708 — Hardware and Architecture

**Work 2.** Real-Time Density--Source Mapping for Open Electron--Photon Systems

> Real-time quantum-electrodynamical density-functional theory (QEDFT) for an open device requires local electron and field histories to determine their sources. For occupation-dependent interactions and a gapped finite-band photon continuum, we prove a causal one-to-one map on the forward image over a nonzero time interval. Three source-reconstruction benchmarks test the electronic limit, the photon-continuum limit, and a coupled joint inverse against a closed-form continuum endpoint. This provides the density--source foundation for real-time open-system QEDFT.

Class path: D3 — Physical Sciences › F17 — Computer Science › S1708 — Hardware and Architecture

## 41. uid `0983c28779`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** A generic construction for high order approximation schemes of semigroups using random grids

> Our aim is to construct high order approximation schemes for general semigroups of linear operators $P_{t},t\geq 0$. In order to do it, we fix a time horizon $T $ and the discretization steps $h_{l}=\frac{T}{n^{l}},l\in \mathbb{N}$ and we suppose that we have at hand some short time approximation operators $Q_{l}$ such that $P_{h_{l}}=Q_{l}+O(h_{l}^{1+\alpha })$ for some $\alpha >0$. Then, we consider random time grids $\Pi (\omega )=\{t_0(\omega )=0<t_{1}(\omega )<...<t_{m}(\omega )=T\}$ such that for all $1\le k\le m$, $t_{k}(\omega )-t_{k-1}(\omega )=h_{l_{k}}$ for some $l_{k}\in \mathbb{N}$, and we associate the approximation discrete semigroup $P_{T}^{\Pi (\omega )}=Q_{l_{n}}...Q_{l_{1}}.$ Our main result is the following: for any approximation order $\nu $, we can construct random grids $\Pi_{i}(\omega )$ and coefficients $c_{i}$, with $i=1,...,r$ such that \[ …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.01 — Applied mathematics, analysis and probability

**Work 2.** An Empirical Study on Channel Effects for Synthetic Voice Spoofing Countermeasure Systems

> Spoofing countermeasure (CM) systems are critical in speaker verification; they aim to discern spoofing attacks from bona fide speech trials. In practice, however, acoustic condition variability in speech utterances may significantly degrade the performance of CM systems. In this paper, we conduct a cross-dataset study on several state-of-the-art CM systems and observe significant performance degradation compared with their single-dataset performance. Observing differences of average magnitude spectra of bona fide utterances across the datasets, we hypothesize that channel mismatch among these datasets is one important reason. We then verify it by demonstrating a similar degradation of CM systems trained on original but evaluated on channel-shifted data. Finally, we propose several channel robust strategies (data augmentation, multi-task learning, adversarial learning) for CM systems, …

Class path: 01 — Computing, AI and engineering

## 42. uid `c5d9d5ffdd`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** Hybrid Improper Ferroelectricity and Moir\'e Superlattices-induced Exciton Quantization in Layered 2D Halide Perovskite

> 2D Ruddlesden-Popper perovskites are compelling platforms for quantum-confined optoelectronics. However, polar order in iodide composition remains rare under ambient conditions, and the mechanistic origin of anomalous photoluminescence in this class of perovskite is still speculative. Here, we demonstrate that solution-grown $(PA)_2FAPb_2I_7$ single crystals develop an inadvertent moir\'e superlattice through pseudo-merohedral twinning, driven by hybrid improper ferroelectricity in which trilinear mode coupling between two primary zone-boundary modes ($X_2^+$ and $X_3^-$) and a secondary $\Gamma_4^-$ polar displacement simultaneously breaks inversion symmetry and imposes a ca. 5.17{\deg} rotational misalignment between adjacent layers. This symmetry breaking activates one of the highest piezoelectric coefficients $d_{33}$ (ca. 20 pm/V) reported among 2D perovskites. This misalignment …

Class path: a — forms › an — quantities › ann — units › annq — two › annqU — e

**Work 2.** Diversity of Ionized Gas Structures in Nearby Metal-poor Dwarf Galaxies

> We investigate whether optical and far-infrared [O III] emission from nearby metal-poor dwarf galaxies can be represented by a homogeneous one-zone ionized-gas model with a single electron temperature and density. Our sample comprises five galaxies from the Herschel Dwarf Galaxy Survey: HS1222+3741, SBS0335-052E, POX186, Haro11, and IZw18. We combine galaxy-integrated or nearly galaxy-integrated [O III] 4363 and 5007 measurements from Seimei/KOOLS-IFU observations and published or archival spectroscopy with Herschel/PACS [O III] 88um measurements. Because [O III] 4363 is not detected in HS1222+3741, the analysis is based on the remaining four galaxies. SBS0335-052E and Haro11 lie near or slightly beyond the low-density boundary of the one-zone diagnostic. Their nominal line ratios favor effective densities of ne<1cm-3, while conservative treatment of the uncertainties allows values up …

Class path: e — atoms › ec — period 2 elements › ecq — oxygen › ecqa — attributes of #ecqa › ecqam — O--

## 43. uid `0a86e5309b`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** Disentangling Semantic Attention from Structural Bias in the Attention Manifold

> The empirical success of attention mechanism in Multimodal Large Language Models (MLLMs) often obscures its inherent, subtle flaws. Specifically, MLLMs consistently exhibit disproportionate attention toward certain semantically uninformative visual tokens, a phenomenon termed "register" or "Visual Attention Sinks." While existing inference intervention methods attempt to identify these sink tokens and redistribute their attention weights, such approaches typically treat these tokens in isolation and suffer from computational inefficiency. Instead, we reframe this phenomenon as a generalized textual bias exerted over visual features that extends beyond isolated sink tokens. From this perspective, a pervasive structural bias leads to the dilution of the semantic visual signal, precipitating multimodal hallucinations as the model prioritizes linguistic priors over valid visual evidence. To …

Class path: p — consciousness › po — cognition › pod — attention

**Work 2.** The Hidden Evolution of Disguised Visual Context inside the VLM

> Visual tokens enter Large Language Models (LLMs) as raw, foreign signals. How they are transformed into meaningful representations and interact with the language space depends entirely on the integration architecture. Whether by treating visual tokens as in-context prompts within the input sequence or injecting them directly into the LLM's intermediate layers. A controlled comparison and understanding of how these architectural choices affect visual information and its internal transformation to integrate with the LLM remains underexplored. We provide a fair comparison by evaluating in-context and layer-wise injection VLM integration paradigms under identical training conditions across single image, multi-image, and video benchmarks. In doing so, we uncover a hidden evolution where visual tokens enter the LLM as disguised visual context, raw representations lacking linguistic structure, …

Class path: q — language › qa — attributes of #qa › qac — language change

## 44. uid `3b06178035`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Quantum entanglement and Bell nonlocality in top-quark pair production at a photon linear collider

> A photon linear collider, the two-photon collision mode of an $e^+e^-$ linear collider, uses high-energy laser photons backscattered off the incoming electrons and positrons. The colliding-photon polarization is fully controllable through the polarizations of the initial electron and positron beams and laser photons. We investigate the impact of colliding-photon polarization on the observability of quantum entanglement in top-quark pair production at a photon linear collider. Constructing the spin density matrix of the $t\bar{t}$ two-qubit system from the helicity amplitudes, we demonstrate that a photon linear collider is an ideal machine to probe quantum entanglement and Bell nonlocality across the broad phase space of the process.

Class path: Q — Science › QC — Physics › QC170-197 — Atomic physics. Constitution and properties of matter › QC174.17.A-174.17.Z — Special topics, A-Z › QC174.17.E58 — Entanglement

**Work 2.** A Semi-Lagrangian Spherical Essentially Non-Oscillatory (SENO) Scheme for Advection Equations of S2-valued Functions

> We develop a numerical scheme for solving the advection equation of $\mathbb{S}^2$-valued functions of real variables, which models the time-evolution of a $\mathbb{S}^2$-valued mapping on the real line by a known velocity field. The idea is to extend the semi-Lagrangian method for the linear scalar advection equation. We first construct the backward flow map between two adjacent time levels and then interpolate the discrete ordered data of $\mathbb{S}^2$. To handle $\mathbb{S}^2$-functions which have kinks or sharp discontinuity in their components, we incorporate the \textit{Spherical Essentially Non-Oscillatory} (SENO) interpolation method, which effectively reduces the spurious oscillations in high-order reconstructions. We will show multiple examples to demonstrate the accuracy and effectiveness of the proposed algorithm for the partial differential equation of …

Class path: Z — Bibliography. Library science. Information resources (General) › Z5051-7999 — Subject bibliography › Z6651-6655 — Mathematics (Table Z5 modified) › Z6654.A-6654.Z — Special topics, A-Z › Z6654.F9 — Functions

## 45. uid `6a44c48b07`

Scheme: UNESCO Thesaurus

**Work 1.** APEIRON: composing smart TDAQ systems for high energy physics experiments

> We present APEIRON, a distributed heterogeneous processing framework comprising both hardware architecture and software stack for multi-FPGA systems. Targeting smart trigger and data acquisition (TDAQ) systems in high energy physics, APEIRON spans the full software hierarchy: from low-level device drivers to a high-level dataflow programming model based on High-Level Synthesis. We describe the framework design, its core communication infrastructure, and a particle identification application for the NA62 experiment as a representative physics use case.

Class path: 2 — Science › 2.20 — Physical sciences › c132 — Physics

**Work 2.** Regularization independent of the noise level: an analysis of quasi-optimality

> The quasi-optimality criterion chooses the regularization parameter in inverse problems without taking into account the noise level. This rule works remarkably well in practice, although Bakushinskii has shown that there are always counterexamples with very poor performance. We propose an average case analysis of quasi-optimality for spectral cut-off estimators and we prove that the quasi-optimality criterion determines estimators which are rate-optimal {\em on average}. Its practical performance is illustrated with a calibration problem from mathematical finance.

Class path: 2 — Science › 2.60 — Pollution, disasters and safety › c207 — Environmental degradation › c6445 — Pollution › c6456 — Noise pollution

## 46. uid `64e8479f56`

Scheme: UNESCO Thesaurus

**Work 1.** At a Glance to Your Fingertips: Enabling Direct Manipulation of Distant Objects Through SightWarp

> In 3D user interfaces, reaching out to grab and manipulate something works great until it is out of reach. Indirect techniques like gaze and pinch offer an alternative for distant interaction, but do not provide the same immediacy or proprioceptive feedback as direct gestures. To support direct gestures for faraway objects, we introduce SightWarp, an interaction technique that exploits eye-hand coordination to seamlessly summon object proxies to the user's fingertips. The idea is that after looking at a distant object, users either shift their gaze to the hand or move their hand into view-triggering the creation of a scaled near-space proxy of the object and its surrounding context. The proxy remains active until the eye-hand pattern is released. The key benefit is that users always have an option to immediately operate on the distant object through a natural, direct hand gesture. …

Class path: 6 — Politics, law and economics › 6.70 — Finance and trade › c664 — Trade › c13608 — Markets › c10308 — Market structure › c6377 — Consumers

**Work 2.** WarpHammer: Densifying Scene Warps with 3D Object Priors for Extreme View Synthesis

> Projection-conditioned novel view synthesis (NVS) warps an explicit 3D reconstruction of the input view into the target camera and conditions a generator on the warped rendering. This works well for small viewpoint changes but degrades sharply under large orbital motion: the warp becomes sparse around the orbited object, where hidden surfaces dominate the new view and mirror-like artifacts emerge, causing the generator to lose both pixel content and the implicit camera cue carried by the warp. We introduce WarpHammer, a training-free framework that resolves this failure mode by augmenting the warped scene with an explicit 3D reconstruction of the object obtained from a native 3D generative prior (e.g., SAM3D). The reconstructed object adds missing foreground surfaces and occludes background points that should no longer be visible, restoring both appearance and camera cues without …

Class path: 6 — Politics, law and economics › 6.30 — Economic and social development › c594 — Economic and social development › c1800 — Development policy › c17049 — Reconstruction

## 47. uid `f3493a2242`

Scheme: UNESCO Thesaurus

**Work 1.** Towards Reliable Aerial Ground Vehicle Collaboration: An Integrated Planning and Autonomy Framework for Field Deployment

> Limited flight endurance significantly restricts the operational range of unmanned aerial vehicles (UAVs) in long duration missions such as surveillance and inspection, where multiple spatially distributed Areas of Interest (AOIs) must be visited. These tasks require efficient routing determining the sequence of visits which directly impacts mission time, energy consumption, and overall feasibility. Pairing UAVs with unmanned ground vehicles (UGVs) for mobile recharging offers a promising solution, but introduces a tightly coupled cooperative routing problem involving UAV route planning, UGV road constrained movement, energy management, and rendezvous scheduling under uncertainty. In this work, we present an integrated planning and autonomy framework for reliable field deployment. We formulate the problem as an energy constrained cooperative routing task and solve it using a Deep …

Class path: 6 — Politics, law and economics › 6.75 — Organization and management › c671 — Management operations › c13687 — Planning › c14053 — Planning methods › c12717 — Integrated planning

**Work 2.** Transfer to Sky: Unveil Low-Altitude Route-Level Radio Maps via Ground Crowdsourced Data

> The expansion of the low-altitude economy is contingent on reliable cellular connectivity for unmanned aerial vehicles (UAVs). A key challenge in pre-flight planning is predicting communication link quality along proposed and pre-defined routes, a task hampered by sparse measurements that render existing radio map methods ineffective. This paper introduces a transfer learning framework for high-fidelity route-level radio map prediction. Our key insight is to leverage abundant crowdsourced ground signals as auxiliary supervision. To bridge the significant domain gap between ground and aerial data and address spatial sparsity, our framework learns general propagation priors from simulation, performs adversarial alignment of the feature spaces, and is fine-tuned on limited real UAV measurements. Extensive experiments on a real-world dataset from Meituan show that our method achieves over …

Class path: 5 — Information and communication › 5.20 — Information industry › c478 — Broadcasting › c3356 — Radio

## 48. uid `62015f907a`

Scheme: arXiv category taxonomy

**Work 1.** Deep D-bar: Real time Electrical Impedance Tomography Imaging with Deep Neural Networks

> The mathematical problem for Electrical Impedance Tomography (EIT) is a highly nonlinear ill-posed inverse problem requiring carefully designed reconstruction procedures to ensure reliable image generation. D-bar methods are based on a rigorous mathematical analysis and provide robust direct reconstructions by using a low-pass filtering of the associated nonlinear Fourier data. Similarly to low-pass filtering of linear Fourier data, only using low frequencies in the image recovery process results in blurred images lacking sharp features such as clear organ boundaries. Convolutional Neural Networks provide a powerful framework for post-processing such convolved direct reconstructions. In this study, we demonstrate that these CNN techniques lead to sharp and reliable reconstructions even for the highly nonlinear inverse problem of EIT. The network is trained on data sets of simulated …

Class path: grp_math — Mathematics › math — Mathematics › math.NA — Numerical Analysis

**Work 2.** Efficient methods for time-dependence in semiclassical Schr\"odinger equations

> We build an efficient and unitary (hence stable) method for the solution of the semi-classical Schr\"odinger equation subject with explicitly time-dependent potentials. The method is based on a combination of the Zassenhaus decomposition (Bader, Iserles, Kropielnicka & Singh 2014) with the Magnus expansion of the time-dependent Hamiltonian. We conclude with numerical experiments.

Class path: grp_math — Mathematics › math — Mathematics › math.NA — Numerical Analysis

## 49. uid `d81a3963e1`

Scheme: UNESCO Thesaurus

**Work 1.** Identification and Estimation of Staggered Difference-in-Differences with Network Spillovers

> This paper studies staggered difference-in-differences designs in which treated and comparison units can be exposed to other units' adoption. Using an exposure mapping specified by the researcher, we define realized and counterfactual exposure states and decompose the total effect into a switching effect of own adoption and a spillover effect under no own adoption. Conditional parallel trends identify the total and switching effects by matching, respectively, the counterfactual and realized exposure states of the treated cohort to the same states among units that never adopt. The difference between these effects identifies the spillover effect for the treated cohort. We construct outcome regression estimators for these effects and doubly robust estimators for the total and switching effects, and establish their asymptotic normality under spatial dependence. Monte Carlo simulations …

Class path: 4 — Social and human sciences › 4.25 — Social policy and welfare › c415 — Social welfare › c3916 — Child welfare › c1466 — Adoption

**Work 2.** From level set evolution to threshold optimization: A grayscale level set framework for image segmentation

> The segmentation of multiple degradations has been a challenging problem in the field of image segmentation. Existing level set approaches commonly adopt a length regularization term to constrain the geometric shape of the segmentation contour. However, the introduction of the length term often results in numerical instability and high computational cost. In this paper, we show that the length term is not essential under certain smoothness constraints, and theoretically prove that the presence of the length term affects the property of $|\nabla \phi|=1$. Based on the finding, we define a class of smooth images, construct the grayscale level set, and propose a fast segmentation framework for degraded images, such as heavily noisy images and intensity inhomogeneous images. The framework transforms PDE evolution into one-dimensional threshold search, which has significant advantages in …

Class path: 2 — Science › 2.70 — Biology › c223 — Genetics › c4044 — Evolution

## 50. uid `cc54ac46fb`

Scheme: UNESCO Thesaurus

**Work 1.** Elimination of angular dependency in the quantum three-body problem made easy

> We present a systematic account of the separation of the angular degrees of freedom from the nonrelativistic Schr\"{o}dinger equation for a three-body quantum system with arbitrary masses, charges, total angular momentum, and parity. The resulting reduced Schr\"{o}dinger equation (RSE) for the partial-wave components, expressed as functions solely of the interparticle distances, is reported in a compact matrix operator form. The remnants of the angular dependence, essential for the hermiticity of the RSE and consequently the stability of variational computations, appear in the RSE formalism as additional angular factors, derived by expanding minimal bipolar harmonics into a basis of Wigner functions \texorpdfstring{$\mathcal{D}$}{D}. We validate the final form of the RSE by computing accurate energy levels for helium states using an explicitly correlated Hylleraas-type basis. This work …

Class path: 3 — Culture › 3.40 — Literature › c328 — Literature

**Work 2.** A Lay User Explainable Food Recommendation System Based on Hybrid Feature Importance Extraction and Large Language Models

> Large Language Models (LLM) have experienced strong development in recent years, with varied applications. This paper uses LLMs to develop a post-hoc process that provides more elaborated explanations of the results of food recommendation systems. By combining LLM with a hybrid extraction of key variables using SHAP, we obtain dynamic, convincing and more comprehensive explanations to lay user, compared to those in the literature. This approach enhances user trust and transparency by making complex recommendation outcomes easier to understand for a lay user.

Class path: 3 — Culture › 3.40 — Literature › c328 — Literature

## 51. uid `b1c2faf5dd`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Rule Extraction in Machine Learning: Chat Incremental Pattern Constructor

> Rule extraction is a central problem in interpretable machine learning because it seeks to convert opaque predictive behavior into human-readable symbolic structure. This paper presents Chat Incremental Pattern Constructor (ChatIPC), a lightweight incremental symbolic learning system that extracts ordered token-transition rules from text, enriches them with definition-based expansion, and constructs responses by similarity-guided candidate selection. The system may be viewed as a rule extractor operating over a token graph rather than a conventional classifier. I formalize the knowledge base, definition expansion, candidate scoring, repetition control, English-rule heuristics, and response construction mechanisms used by ChatIPC. I further situate the method within the literature on rule extraction, decision tree induction, association rules, interpretable machine learning, and sequence …

Class path: 68-XX — Computer science › 68Txx — Artificial intelligence › 68T05 — Learning and adaptive systems in artificial intelligence

**Work 2.** GeoTop: Advancing Image Classification with Geometric-Topological Analysis

> A fundamental challenge in diagnostic imaging is the phenomenon of topological equivalence, where benign and malignant structures share global topology but differ in critical geometric detail, leading to diagnostic errors in both conventional and deep learning models. We introduce GeoTop, a mathematically principled framework that unifies Topological Data Analysis (TDA) and Lipschitz-Killing Curvatures (LKCs) to resolve this ambiguity. Unlike hybrid deep learning approaches, GeoTop provides intrinsic interpretability by fusing the capacity of persistent homology to identify robust topological signatures with the precision of LKCs in quantifying local geometric features such as boundary complexity and surface regularity. The framework's clinical utility is demonstrated through its application to skin lesion classification, where it achieves a consistent accuracy improvement of 3.6% and …

Class path: 68-XX — Computer science › 68Txx — Artificial intelligence › 68T05 — Learning and adaptive systems in artificial intelligence

## 52. uid `ccb72f0a79`

Scheme: Free Decimal Correspondence

**Work 1.** TRAM: A Transverse Relaxation Time-Aware Qubit Mapping Algorithm for NISQ Devices

> Noisy intermediate-scale quantum (NISQ) devices impose dual challenges on quantum circuit execution: limited qubit connectivity requires extensive SWAP-gate routing, while time-dependent decoherence progressively degrades quantum information. Existing qubit mapping algorithms optimize for hardware topology and static calibration metrics but systematically neglect transverse relaxation dynamics (T2), creating a fundamental gap between compiler decisions and evolving noise characteristics. We present TRAM (Transverse Relaxation Time-Aware Qubit Mapping), a coherence-guided compilation framework that elevates decoherence mitigation to a primary optimization objective. TRAM integrates calibration-informed community detection to construct noise-resilient qubit partitions, generates time-weighted initial mappings that anticipate coherence decay, and dynamically schedules SWAP operations to …

Class path: 500 — Science; Natural history › 510 — Mathematics › 514 — Trigonometry; Topology

**Work 2.** Generation of Coherent Quantum Light from a Single Impurity-Bound Exciton

> Impurity-bound excitons in II-VI direct-bandgap semiconductors are promising optically active solid-state spin qubits that combine exceptional optical quantum efficiency with an ultra-low spin noise environment. Previous studies on single impurities relied on incoherent optical excitation to generate photons. However, many quantum applications require resonant driving of quantum emitters to precisely control optical transitions and maintain coherence of the emission. Here, we demonstrate coherent optical emission of quantum light from a resonantly driven single impurity-bound exciton in ZnSe. The resonantly driven emitter exhibits bright quantum light emission that preserves the phase of the resonant drive, validated through polarization interferometry. Resonant excitation enables us to directly measure the Debye-Waller factor, determined to be 0.94, which indicates high efficiency …

Class path: 500 — Science; Natural history › 530 — Physics › 535 — Optics; Light

## 53. uid `b483ba4748`

Scheme: UNESCO Thesaurus

**Work 1.** Aligning What LLMs Do and Say: Towards Self-Consistent Explanations

> Large language models (LLMs) seem to offer an easy path to interpretability: just ask them to explain their answers. Yet the features driving an answer often differ from those emphasized in its explanation, meaning post-hoc rationales can misrepresent what actually shaped the model's output. We quantify this gap by comparing the feature-importance distributions of answers and their explanations. Prior analyses reveal such discrepancies, but large-scale study has been limited by the high computational cost of attribution methods. To address this, we introduce the Post-hoc Self-Consistency Bank (PSCB), a large-scale benchmark linking model decisions with diverse explanations and attribution vectors across datasets, methods, and model families. Using PSCB, we find that Spearman rank correlation provides a more reliable signal of alignment than cosine similarity. Building on this insight, …

Class path: 2 — Science › 2.15 — Mathematics and statistics › c119 — Statistics › c13824 — Statistical mathematics › c2238 — Statistical analysis › c6658 — Correlation

**Work 2.** Psychologically Potent, Computationally Invisible: LLMs Generate Social-Comparison-Eliciting Posts They Fail to Detect

> We introduce Xiaohongshu Social Comparison Reader Elicitation (XHS-SCoRE), a reader-grounded benchmark for detecting whether text-only Xiaohongshu (RedNote) posts elicit Upward, Downward, or Neutral/no clear social comparison from a first-person reader perspective. The task targets a socially meaningful relational, behaviorally real signal not reducible to sentiment. Across prompted LLM classifiers and supervised Chinese encoders, we find a consistent generation-detection mismatch: the signal is textually learnable in-domain, but not robustly accessible to prompt-based classification. Prompted LLM classifiers show stable failures, especially neutralization of comparison-eliciting posts and model-specific directional skew. A controlled pilot shows that LLM-generated Xiaohongshu-style posts can shift perceived standing and comparison-related affect even when prompt-based detection of the …

Class path: 3 — Culture › 3.35 — Languages › c320 — Asian languages › c3142 — South and Southeast Asian languages › c5119 — Chinese

## 54. uid `ccaafc8a89`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** EidosDoc: Implicit Structure Encoding for Cost-Effective Semi-Structured Document QA

> Semi-structured documents are ubiquitous in scientific reports, financial statements, and technical manuals. Question answering over such documents requires simultaneous understanding of text, tables, charts, and complex hierarchical layouts. Existing methods either rely on repeatedly calling large language models for structure parsing and retrieval, leading to high cost and large latency, or they flatten the document and lose layout and hierarchy information, sacrificing answer accuracy. To address this, we propose EidosDoc, a novel system that achieves state-of-the-art accuracy with minimal computational expense. Our approach introduces three core innovations. (1) An Implicit Structure Encoder trained via contrastive learning and a structure consistency loss. This module jointly embeds hierarchical relationships, spatial positions, and textual content into a dense vector space, …

Class path: r — rituals › ra — attributes of #ra › rab — historical periods › rabX — millennia › rabXX — centuries › rabXXX — decades › rabXXXX — years › rabXXXXX — months › rabXXXXXX — dates › rabXXXXXXX — times

**Work 2.** UltRAG: a Universal Simple Scalable Recipe for Knowledge Graph RAG

> Large language models (LLMs) frequently generate confident yet factually incorrect content when used for language generation (a phenomenon often known as hallucination). Retrieval augmented generation (RAG) tries to reduce factual errors by identifying information in a knowledge corpus and putting it in the context window of the model. While this approach is well-established for document-structured data, it is non-trivial to adapt it for Knowledge Graphs (KGs), especially for queries that require multi-node/multi-hop reasoning on graphs. We introduce UltRAG, a training-free KG-RAG recipe that combines LLM query generation, a fully inductive neural query executor, and LLM arbitration. This off-the-shelf composition achieves state-of-the-art results on Knowledge Graph Question Answering (KGQA) tasks without retraining the LLM or executor, while enabling language models to interface with …

Class path: y — knowledge › yt — documents › ytq — models

## 55. uid `bb1ceecbe0`

Scheme: OpenAlex Topics

**Work 1.** Before the Tool Call: Deterministic Pre-Action Authorization for Autonomous AI Agents

> AI agents today have passwords but no permission slips. They execute tool calls (fund transfers, database queries, shell commands, sub-agent delegation) with no standard mechanism to enforce authorization before the action executes. Current safety architectures rely on model alignment (probabilistic, training-time) and post-hoc evaluation (retrospective, batch). Neither provides deterministic, policy-based enforcement at the individual tool call level. We characterize this gap as the pre-action authorization problem and present the Open Agent Passport (OAP), an open specification and reference implementation that intercepts tool calls synchronously before execution, evaluates them against a declarative policy, and produces a cryptographically signed audit record. OAP enforces authorization decisions in a measured median of 53 ms (N=1,000). In a live adversarial testbed (4,437 …

Class path: D3 — Physical Sciences › F17 — Computer Science › S1702 — Artificial Intelligence

**Work 2.** A Survey on Agentic Security: Applications, Threats and Defenses

> LLM-based agents are now used throughout cybersecurity. While these agents facilitate powerful and autonomous security applications, their autonomy opens up new attack surfaces, and the security community is actively building defenses to secure them. Yet the literature on this subject has grown quickly and unevenly. Existing surveys treat applications, threats, and defenses in isolation, leaving no unified account of how an agent's capabilities, vulnerabilities, and countermeasures interconnect. In this work we present the first holistic survey of the agentic security landscape, structuring the field around the fundamental pillars of Applications, Threats and Defenses. We provide a comprehensive taxonomy of over 260 papers, explaining how agents are used in downstream cybersecurity applications, inherent threats to agentic systems, and countermeasures designed to protect them. In …

Class path: D2 — Social Sciences › F14 — Business, Management and Accounting › S1408 — Strategy and Management

## 56. uid `1ce1d8fca2`

Scheme: arXiv category taxonomy

**Work 1.** Deep learning methods for stochastic Galerkin approximations of random domain problems

> This work considers strong and weak stochastic Galerkin approximations of random domain problems for the elliptic random partial differential equation (PDE). The random domain problems are realized by sufficiently regular random domain mapping, allowing a transformation on deterministic reference domain. A traditional numerical method for solving the resulting high-dimensional coupled stochastic Galerkin systems is replaced by deep learning techniques. We compare a physics-informed neural network approach, based on the strong stochastic Galerkin residual, with a Deep Ritz approach, based on the weak stochastic Galerkin system, reformulated as Ritz energy minimization. The neural networks serve as surrogates for the deterministic spectral coefficients of the respective stochastic Galerkin solution. The stochastic parameters are not used as neural network input parameters and the …

Class path: grp_math — Mathematics › math — Mathematics › math.NA — Numerical Analysis

**Work 2.** Convergence of Stochastic Gradient Methods for Wide Two-Layer Physics-Informed Neural Networks for the Poisson Equation

> Physics informed neural networks (PINNs) represent a very popular class of neural solvers for partial differential equations. In practice, one often employs stochastic gradient descent type algorithms to train the neural network. Therefore, the convergence guarantee of stochastic gradient descent is of fundamental importance. In this work, we establish the linear convergence of stochastic gradient descent / flow in training over-parameterized two layer PINNs with a general class of activation functions for solving one model second-order elliptic problem, i.e., the Poisson equation, in the sense of high probability. These results extend the existing result [20] in which gradient descent was analyzed. The challenge of the analysis lies in handling the dynamic randomness introduced by stochastic optimization methods. The key of the analysis lies in ensuring the positive definiteness of …

Class path: grp_cs — Computer Science › cs — Computer Science › cs.LG — Machine Learning

## 57. uid `3240fd9b01`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Convex Networks Remain Hard to Certify: Dimension-Accuracy Barriers for Lipschitz Constants

> Input-convex neural networks permit globally tractable minimization over their inputs, so one might expect their global regularity to be tractable in low input dimension. We prove exact and accuracy-sensitive barriers to this expectation. Given a bias-free one-hidden-layer ReLU network $f(x)=\sum_{r=1}^n \mathrm{ReLU}(a_r^\top x)$ with unit positive output weights, deciding whether its global Euclidean Lipschitz constant is at least a rational threshold is NP-complete and W[1]-hard when parameterized by the input dimension $d$. The same holds on the unit ball and with integral first-layer weights having at most nine nonzeros. More sharply, no deterministic multiplicative approximation scheme runs in $g(d)\mathrm{poly}(\mathcal B,1/\varepsilon)$ time unless FPT equals W[1]. Under the Exponential Time Hypothesis, no such algorithm runs in $g(d)(\mathcal B+1/\varepsilon)^{o(d/\log d)}$ …

Class path: B — Philosophy. Psychology. Religion › BQ — Buddhism › BQ840-999 — Biography › BQ860-999 — Individual › BQ940-999 — Other individuals › BQ950 — D

**Work 2.** Gate Control of g-factor in Germanium Quantum Dots: A Strain-Based Explanation

> The g-factor is a key parameter governing the behavior of semiconductor spin qubits, as it directly determines the qubit frequency and its sensitivity to electrical and magnetic noise. Recent experiments in germanium quantum dots have revealed large g-factor variations under small gate voltage changes, indicating a strong coupling between electrostatics and spin properties. Here, we present a quantitative explanation based on strain-induced g-tensor modulation. By combining finite-element simulations of inhomogeneous strain with quantum calculations of hole wavefunctions, we show that device-induced strain produces spatially varying g-tensors. Gate voltages shift the quantum dot within this landscape, leading to substantial changes in the effective g-factor. Our results may account for the experimentally observed tunability and highlight the importance of in-plane g-tensor variations. …

Class path: B — Philosophy. Psychology. Religion › BQ — Buddhism › BQ840-999 — Biography › BQ860-999 — Individual › BQ940-999 — Other individuals › BQ960 — G

## 58. uid `5b9cbd6bd9`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** The Disciplinary Language Transfer Problem: How Psychological Vocabulary Produces Governance Failures in AI Agent Deployment

> The vocabulary used to describe AI agents in governance contexts -- learning, memory, values, compliance, identity, trust -- is borrowed from psychological and organizational science, contributing to systematic failures in how organizations deploy, oversee, and hold agents accountable. This paper argues that the problem is not merely terminological but epistemological: psychological vocabulary carries an "invisible grammar" of its home discipline into governance discourse, calibrating frameworks to a metaphysical entity that does not exist in current AI architectures. We call this the disciplinary language transfer problem. Drawing on Wittgenstein's concept of language games, Kuhn's paradigm-laden observation, Haraway's situated knowledge, and Star and Griesemer's boundary object theory, we show that the transfer operates at three levels (epistemological assumptions, theoretical …

Class path: 01 — Computing, AI and engineering › 01.01 — Language models, security and computing systems › 01.01.01 — Language, speech and LLM agents › 01.01.01.02 — LLM agents

**Work 2.** Magellan: Autonomous Discovery of Novel Compiler Optimization Heuristics with AlphaEvolve

> Modern compilers rely on hand-crafted heuristics to guide optimization passes. These human-designed rules often struggle to adapt to the complexity of modern software and hardware and lead to high maintenance burden. To address this challenge, we present Magellan, an agentic framework that evolves the compiler pass itself by synthesizing executable C++ decision logic. Magellan couples an LLM coding agent with evolutionary search and autotuning in a closed loop of generation, evaluation on user-provided macro-benchmarks, and refinement, producing compact heuristics that integrate directly into existing compilers. Across several production optimization tasks, Magellan discovers policies that match or surpass expert baselines. In LLVM function inlining, Magellan synthesizes new heuristics that outperform decades of manual engineering for both binary-size reduction and end-to-end …

Class path: 01 — Computing, AI and engineering › 01.01 — Language models, security and computing systems › 01.01.01 — Language, speech and LLM agents › 01.01.01.02 — LLM agents

## 59. uid `d92425c9dd`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** Four-digit Kaprekar dynamics in odd bases

> Start with four digits, arrange them in both descending and ascending order, subtract, and repeat. This simple process is known as the Kaprekar routine, famous in base ten for sending every nonconstant four-digit string to $6174$. We show that in every odd base $B>3$, the four-digit Kaprekar map has an unexpectedly rigid structure. After at most three iterations, every nonconstant orbit enters an explicit triangular region $\mathcal{T}_B$, and on this region the map is conjugate to projective doubling: \[ \{[r],[s]\}\longmapsto \{[2r],[2s]\}. \] This gives a complete finite description of all nonconstant terminal cycles, including an explicit formula for their lengths and counts. In particular, the longest terminal cycle has length at most $(B-1)/2$, and equality can occur only when $B$ is prime. For primes $p>5$, equality occurs precisely when the least positive $m$ with …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics

**Work 2.** Kinematics of Single-Winged Spinning Seeds: A Study on Mahogany and Buddha Coconut Samaras

> This study investigates the steady-state kinematics of single-winged spinning samaras and re-evaluates the simplifying assumptions commonly used in existing theoretical models. High-speed imaging was employed to quantify key parameters, including descent velocity, rotational speed, coning angle, pitching motion, and the precessional trajectory of the center of mass. The results show that all measured parameters exhibit significant temporal variation, contradicting the long-standing assumption that these quantities remain constant in steady-state flight. This variability reveals that commonly used steady-state simplifications in previous studies may overlook essential aerodynamic mechanisms governing natural samara descent. Despite this complexity, the observed sinusoidal variations in pitch, cone angle, and translational velocity, together with the nearly linear rotation rate, provide a …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.01 — Applied mathematics, analysis and probability › 02.01.01.01 — Convex and nonconvex optimization

## 60. uid `7d6a015a30`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** Quantum entanglement between partons in a strongly coupled quantum field theory

> We perform a first-principles, non-perturbative investigation of quantum entanglement between partonic constituents in a strongly coupled 3+1-dimensional scalar Yukawa theory, using light-front Hamiltonian methods with controlled Fock-space truncations. By explicitly constructing reduced density matrices for (mock) nucleon, pion, and anti-nucleon subsystems from light-front wave functions, we compute key entanglement witnesses, including von Neumann entropy, mutual information, and linear entropy, in both quenched (no sea pairs) and unquenched frameworks. We find that the entanglement entropy is closely related to the Shannon entropy of the transverse momentum dependent distribution, establishing a link between quantum information and parton structure. In contrast, the unquenched theory reveals genuinely non-classical correlations: the entanglement entropy cannot be reduced to any …

Class path: g94 — at K temperature › g944 — entropy

**Work 2.** A Single Conformal Coordinate Decodes Outcome-Resolved Measurement-Induced Entanglement

> A local measurement of a critical many-body state produces exponentially many microscopic outcomes, each defining a different conditioned quantum state. We ask how much information from one such outcome is actually needed to predict the entanglement that remains between two unmeasured regions. We show that, in critical quantum chains, the universal outcome-resolved R\'enyi entropy is asymptotically determined by a single compact conformal coordinate extracted directly from the measurement record. Compact-boson theory explains this compression: the universal entropy is insensitive to all spatially varying components of the boundary profile, retaining only a single relative compact zero mode. At the free-fermion XX point, exact lattice identities expose the microscopic structure underlying the decoder, while large-scale outcome-by-outcome calculations confirm its scaling prediction across …

Class path: a — forms › an — quantities › ann — units › annp — one

## 61. uid `331dc6393e`

Scheme: arXiv category taxonomy

**Work 1.** A Family of Even-Order Central-Upwind WENO Schemes with Averaged Downwind and Novel Global Smoothness Indicators

> We propose a simple yet effective local smoothness indicator for the downwind stencil in central-upwind weighted essentially non-oscillatory (WENO) schemes of even order for hyperbolic conservation laws. Starting from an odd-order upwind WENO scheme, we construct an even-number-of-points stencil by incorporating a downwind substencil whose smoothness indicator is the arithmetic mean of all local smoothness indicators. This straightforward averaging approach incorporates regularity information from the entire stencil without requiring additional tuning parameters or complex formulations. Combined with affine-invariant Z-type nonlinear weights and a carefully designed global smoothness indicator, the resulting scheme, termed WENO-ZA6 for the sixth-order case, achieves optimal convergence rates at critical points up to second order, exhibits favorable dispersion and dissipation properties …

Class path: grp_math — Mathematics › math — Mathematics › math.NA — Numerical Analysis

**Work 2.** A Novel Compressed Sensing Scheme for Photoacoustic Tomography

> Speeding up the data acquisition is one of the central aims to advance tomographic imaging. On the one hand, this reduces motion artifacts due to undesired movements, and on the other hand this decreases the examination time for the patient. In this article, we propose a new scheme for speeding up the data collection process in photoacoustic tomography. Our proposal is based on compressed sensing and reduces acquisition time and system costs while maintaining image quality. As measurement data we use random combinations of pressure values that we use to recover a complete set of pressure data prior to the actual image reconstruction. We obtain theoretical recovery guarantees for our compressed sensing scheme and support the theory by reconstruction results on simulated data as well as on experimental data.

Class path: grp_math — Mathematics › math — Mathematics › math.NA — Numerical Analysis

## 62. uid `268b0c53e3`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Parallel Complexity of Forward and Backward Propagation

> We show that the forward and backward propagation can be formulated as a solution of lower and upper triangular systems of equations. For standard feedforward (FNNs) and recurrent neural networks (RNNs) the triangular systems are always block bi-diagonal, while for a general computation graph (directed acyclic graph) they can have a more complex triangular sparsity pattern. We discuss direct and iterative parallel algorithms that can be used for their solution and interpreted as different ways of performing model parallelism. Also, we show that for FNNs and RNNs with $k$ layers and $\tau$ time steps the backward propagation can be performed in parallel in O($\log k$) and O($\log k \log \tau$) steps, respectively. Finally, we outline the generalization of this technique using Jacobians that potentially allows us to handle arbitrary layers.

Class path: 68-XX — Computer science › 68Txx — Artificial intelligence › 68T05 — Learning and adaptive systems in artificial intelligence

**Work 2.** Majority Vote Silences Minority Values: Annotator Disagreement at the Hate/Offensive Boundary in HateXplain

> Hate speech annotation pipelines routinely collapse annotator disagreement into majority vote labels before training. We show that this aggregation is not neutral: 42.6% of all annotator disagreement in HateXplain concentrates specifically at the hate/offensive boundary, a pattern consistent with annotators applying different thresholds for where hate begins (chi-squared = 135.199, df = 2, p < 0.0001). Both a hard-label BERT model (Model A) and a soft-label model (Model B) drop 22 percentage points in accuracy from agreed posts (~80%) to disagreement posts (~58%), confirmed at p < 0.0001. A per-annotator multi-head model (Model C) widens this gap further to 28 points while collapsing offensive disagreement accuracy to 0.245. Critically, Model A expresses significantly higher confidence on boundary case errors than Model C (0.710 vs. 0.495, p < 0.0001), meaning standard evaluation …

Class path: 68-XX — Computer science › 68Txx — Artificial intelligence › 68T05 — Learning and adaptive systems in artificial intelligence

## 63. uid `138e05b9fd`

Scheme: Free Decimal Correspondence

**Work 1.** Kinetic wave activity and proton heating in 3D hybrid simulations of decaying balanced and imbalanced Alfv\'enic turbulence

> We investigate turbulent dynamics from large to sub-proton scales, the kinetic wave activity and resulting proton heating in three-dimensional hybrid-kinetic simulations of freely decaying balanced and imbalanced Alfv\'enic turbulence. We considered initial fluctuations with same amplitude and spectral shape but different degrees of correlation between velocity and magnetic field perturbations (i.e., normalized cross-helicity), which define the corresponding turbulent regimes. We found that our balanced turbulence simulation exhibits stronger energy dissipation and more efficient proton heating compared to the imbalanced case. On the large fluid scales, the two simulations undergo distinct nonlinear dynamic and energy cascades, leading to different spectral properties once turbulence is well developed. At sub-proton scales, fluctuations are predominantly oblique with right-hand …

Class path: 600 — Industrial arts; Technology; Engineering › 690 — Building › 697 — Heating; Ventilation; Air conditioning

**Work 2.** The role of three-dimensional effects on ion injection and acceleration in perpendicular shocks

> Understanding the conditions that enable particle acceleration at non-relativistic collisionless shocks is essential to unveil the origin of cosmic rays. We employ 2D and 3D hybrid simulations (with kinetic ions and fluid electrons) to explore particle acceleration and magnetic field amplification in non-relativistic perpendicular shocks, focusing on the role of shock drift acceleration and its dependence on the shock Mach number. We perform an analysis of the ion injection process and demonstrate why efficient acceleration is only observed in 3D. In particular, we show that ion injection critically depends on the "porosity" of the magnetic turbulence in the downstream region near the shock, a property describing how easily the post-shock region allows particles to traverse it and return upstream without being trapped. This effect can only be properly captured in 3D. Additionally, we …

Class path: 100 — Philosophy and psychology › 120 — Special metaphysical topics › 123 — Free will and determinism; Necessity (Philosophy)

## 64. uid `5be18dfee6`

Scheme: OpenAlex Topics

**Work 1.** Fast Random Compilers for Time-Dependent and Time-Independent Lindbladian Simulation

> Randomized product formulas provide simple and flexible methods for quantum simulation. While qDRIFT-type randomized simulation has been extended to Lindbladian dynamics, higher-order randomized compilers have primarily been developed for Hamiltonian simulation. In this work, we develop first- and second-order randomized sampling algorithms for Lindbladian simulation in both the time-independent and time-dependent settings. Building on first-order qDRIFT-type Lindbladian simulation, we introduce a qSWIFT-type second-order correction for time-independent Lindbladians with a finite local decomposition. For $\lambda t \geq 1$, where $\lambda$ is the sum of the Hamiltonian coefficients and dissipative rates, the resulting approximation achieves normalized diamond-norm error at most $\varepsilon$ using $r=O(\lambda^2 t^2/\sqrt{\varepsilon})$ time slices, improving the $\varepsilon^{-1}$ …

Class path: D2 — Social Sciences › F14 — Business, Management and Accounting › S1406 — Marketing › T14244 — Consumer Behavior and Marketing Influence

**Work 2.** Learning and simulating bosonic systems via finite-energy locality

> Bosonic devices promise applications in simulation, sensing and quantum error correction, but infinite-dimensional local Hilbert spaces obstruct the locality tools that make qubit dynamics efficiently learnable and simulable. We establish a finite-energy locality principle for geometrically local bosonic open systems satisfying photon-number moment propagation. It compares unbounded generators with Galerkin cutoffs, transferring finite-dimensional Lieb--Robinson, product-formula and circuit techniques to bosons with explicit errors. For this moment-controlled class, we obtain, to our knowledge, the first model-independent weak Lieb--Robinson bounds beyond Bose--Hubbard-type dynamics, together with quantitative Trotter and simulation guarantees for polynomial bosonic GKSL generators. As a central application, coherent-state preparation and local heterodyne detection suffice to learn …

Class path: D3 — Physical Sciences › F21 — Energy

## 65. uid `685c83aab2`

Scheme: OpenAlex Topics

**Work 1.** Topological Obstructions to Dynamical Convexity

> We study the topological obstructions of dynamical convexity on contact manifolds focusing on fillability by cotangent bundles and subcritical surgeries. Using links to algebraic geometry, we motivate and define a stronger version of dynamical convexity, and investigate the topology of these manifolds. More precisely, we show that strongly dynamically convex contact manifolds cannot arise as a unit cotangent bundle $(ST^*M,\lambda_{std})$ of a closed manifold $M$ and in particular that simply connected dynamically convex contact manifolds cannot be filled by cotangent bundles. We demonstrate that dynamical convexity can be used to recover homotopy groups of topologically simple fillings with vanishing symplectic homology. We also show obstructions to dynamical convexity that come from studying different kinds of subcritical surgeries.

Class path: D3 — Physical Sciences › F26 — Mathematics › S2610 — Mathematical Physics › T10896 — Homotopy and Cohomology in Algebraic Topology

**Work 2.** The Impact of the General Data Protection Regulation (GDPR) on Online Usage Behavior

> Privacy regulations aim to safeguard consumers, but can have unintended consequences on how users interact with websites. This article estimates the causal effect of the EU's General Data Protection Regulation (GDPR) on online usage behavior and decomposes it into usage frequency (unique visitors) and usage intensity (visits per unique visitor). Using a generalized synthetic control estimator across trillions of visits to 6,387 websites in 24 industries and 13 countries-11 months before and 19 months after enforcement-it compares observations subject to the GDPR (EU users or EU websites) with unaffected observations (non-EU users on non-EU websites). Weekly visits decline by 4.88% within 3 months and by 10.02%% after 18 months, with the decline increasingly driven by usage frequency: by 18 months, unique visitors decline by 6.61%, whereas visits per unique visitor decline by only 0.59%. …

Class path: D2 — Social Sciences › F33 — Social Sciences › S3308 — Law › T13364 — Digitalization, Law, and Regulation

## 66. uid `100c75b112`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** RTSMS: Randomized Tucker with single-mode sketching

> We propose RTSMS (Randomized Tucker via Single-Mode-Sketching), a randomized algorithm for approximately computing a low-rank Tucker decomposition of a given tensor. It uses sketching and least-squares to compute the Tucker decomposition in a sequentially truncated manner. The algorithm only sketches one mode at a time, so the sketch matrices are significantly smaller than alternative approaches. The algorithm is demonstrated to be competitive with existing methods, sometimes outperforming them by a large margin.

Class path: N — Fine arts › NC — Drawing. Design. Illustration › NC795 — Sketching

**Work 2.** IGV-RRT: Prior-Real-Time Observation Fusion for Active Object Search in Changing Environments

> Object Goal Navigation (ObjectNav) in temporally changing indoor environments is challenging because object relocation can invalidate historical scene knowledge. To address this issue, we propose a probabilistic planning framework that combines uncertainty-aware scene priors with online target relevance estimates derived from a Vision Language Model (VLM). The framework contains a dual-layer semantic mapping module and a real-time planner. The mapping module includes an Information Gain Map (IGM) built from a 3D scene graph (3DSG) during prior exploration to model object co-occurrence relations and provide global guidance on likely target regions. It also maintains a VLM score map (VLM-SM) that fuses confidence-weighted semantic observations into the map for local validation of the current scene. Based on these two cues, we develop a planner that jointly exploits information gain and …

Class path: H — Social sciences › HF — Commerce › HF5001-6182 — Business › HF5546-5548.6 — Office management › HF5548.4.A-5548.4.Z — Special computers, systems, or programs, A-Z › HF5548.4.F7 — Framework

## 67. uid `de4fc98959`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Arithmetic statistics of isogeny Selmer groups associated to hyperelliptic curves

> We determine asymptotic results for the average size of Selmer groups arising from certain isogenies related to Jacobians of hyperelliptic curves of genus $g\geq 2$. Our results come from two different routes. The first route is through the geometry-of-numbers methods pioneered by Bhargava, where we obtain new parametrisations coming from Vinberg theory arising from representations related to the Dynkin diagrams of type $B$ and $C$. The second route uses recent results by Koymans--Smith, which relate the average sizes of Selmer groups to Tamagawa ratios through a formula by Greenberg--Wiles.

Class path: 11-XX — Number theory › 11Gxx — Arithmetic algebraic geometry (Diophantine geometry) › 11G30 — Curves of arbitrary genus or genus \(\ne 1\) over global fields

**Work 2.** Learning General Causal Structures with Hidden Dynamic Process for Climate Analysis

> Understanding climate dynamics requires going beyond correlations in observational data to uncover the underlying causal process. Latent drivers such as atmospheric processes play a central role in temporal dynamics, while direct causal influences also exist among geographically proximate observed variables. Traditional Causal Representation Learning (CRL) typically focuses on latent factors but overlooks such observable-to-observable causal relations, which limits its applicability to climate analysis. In this paper, we introduce a unified framework that jointly uncovers (i) causal relations among observed variables and (ii) latent driving forces together with their interactions. We establish conditions under which both the hidden dynamic process and the causal structure among observed variables are simultaneously identifiable from time-series data, and our guarantees continue to hold …

Class path: 62-XX — Statistics › 62Dxx — Statistical sampling theory and related topics › 62D20 — Causal inference from observational studies

## 68. uid `6f1cf35b08`

Scheme: Free Decimal Correspondence

**Work 1.** Quantitative estimates for singularity for conjugate equations driven by linear fractional transformations

> We consider the conjugate equation driven by two families of finite maps on the unit interval satisfying a compatibility condition. This framework contains de Rham's functional equations. We give sufficient conditions for singularity of the solution with quantitative estimates in the case where the equation is driven by a family of non-affine maps and a family of linear fractional transformations.

Class path: 900 — History; Geography › 910 — Geography; Voyages and travels › 912 — Maps; Atlases

**Work 2.** AlterSeeK-Path: Systematic construction of generalized band-structure paths for displaying altermagnetic spin splitting

> Altermagnetic materials exhibit spin splitting in their electronic band structures while maintaining zero net magnetization. However, conventional high-symmetry k-paths generally hide this splitting because they follow symmetry lines that in most cases enforce spin degeneracy. We present AlterSeeK-Path, an open-source Python tool that systematically constructs generalized band-structure paths for collinear altermagnets. The method selects the centroid of the conventional irreducible wedge used in routine band-structure calculations as the representative general k-point, maps it to a spin-flip-related partner, and inserts paired segments through these points into the standard high-symmetry path; these segments systematically sample the interior of the irreducible wedge. We demonstrate the construction for all 54 three-dimensional combinations of extended Bravais lattice type and spin …

Class path: 900 — History; Geography › 910 — Geography; Voyages and travels › 912 — Maps; Atlases

## 69. uid `4c9237759d`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Eliminating Infinite Self-Energies From Classical Electrodynamics

> The theory of point-particles in classical electrodynamics has a well-known problem of infinite self-energy, and the same is true of quantum electrodynamics. Instead of concluding that there is no such thing as a true point-particle, it is shown here how to remove the infinities by supposing that the electromagnetic field tensor has a symmetric part. This does not change the physics, as the equation of motion and the antisymmetric part of the retarded fields appearing in the equation of motion are unaffected. The symmetric part of the field tensor is not observable and therefore it need not be gauge-invariant, whereas the antisymmetric part is observable, gauge-invariant, and satisfies both the Maxwell Equations and the field equations governing the whole field tensor. This approach goes well beyond prior efforts at classical renormalization, and also entails a new derivation of the …

Class path: Q — Science › QC — Physics › QC501-766 — Electricity and magnetism › QC501-721 — Electricity › QC717.6-718.8 — Plasma physics. Ionized gases › QC718.5.A-718.5.Z — Special topics, A-Z › QC718.5.E4 — Electrodynamics

**Work 2.** A Tilt-Rotor UAV with a Gripper for Stable Contact-Based Tasks via Environmental Anchoring

> Maintaining a stable pose during physical interaction is a significant challenge for aerial robots, often limiting their use in contact-based tasks. This paper presents a novel uncrewed aerial vehicle (UAV) platform designed to transition from unconstrained flight to a stable, constrained work platform via environmental anchoring. Our system comprises: 1) a multirotor with a tilt-rotor mechanism that decouples pitch from forward motion, enabling stable hover at non-zero pitch angles, and 2) a novel underactuated, cable-driven, prismatic gripper featuring compliance to adapt to irregular geometries, designed to stabilize the UAV by anchoring it to its environment. We present the design and prototyping of the complete system and validate its performance through a series of real-robot flight tests. Results demonstrate that anchoring significantly improves stability for interaction tasks, …

Class path: Q — Science › QC — Physics › QC501-766 — Electricity and magnetism › QC501-721 — Electricity › QC717.6-718.8 — Plasma physics. Ionized gases › QC718.5.A-718.5.Z — Special topics, A-Z › QC718.5.D7 — Drift

## 70. uid `da01c333df`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Sampling schemes of multitype continuous-time Bienaym\'e-Galton-Watson trees and limiting critical genealogies

> We study the genealogies of samples of $k$ distinguished particles drawn from the population alive at some fixed time in a continuous-time multitype Bienaym\'e-Galton-Watson (MBGW) process under two different type dependent sampling schemes: uniform sampling without replacement within types given a fixed type configuration, and sampling according to type-dependent weights. These schemes complement the uniform sampling at fixed time $T$ considered in Angtuncio, Pardo, C. Harris (2026a) which did not distinguish between sampled types. Under each scheme for a fixed sampling time $T$, we characterise the associated times of most recent common ancestors, ancestral offspring distributions, and type-dependent ancestral structure of the sample genealogy. In addition, under the assumption that the MBGW process is critical with finite second moments, we show that, conditional on survival of the …

Class path: 05-XX — Combinatorics › 05Cxx — Graph theory › 05C05 — Trees

**Work 2.** Metric representations by minimal graphs

> A resolving set in a graph $G$ is a vertex subset $W= \{\omega^1, \dots, \omega^n\} \subseteq V(G)$ such that each $u \in V(G)$ can be uniquely identified by the vector $r(u \vert W) = (d(u,\omega^1), \dots, d(u,\omega^n))$ of metric coordinates of $u$ with respect to $W$. The reverse problem of identifying the vector sets that are a set of coordinates of some graph provides the concept of realizable vector set $S \subset \mathbb{Z}^n$ by a pair $(G, W)$ meaning that $S=\{ r(u\vert W)\colon u\in V(G)\}$ with $W$ a resolving set of the graph $G$. Here we focus on the minimality of the realizations of vector sets with respect to their edge sets. On the one hand, we study conditions under which it is possible to remove an edge from the graph and keep the realizability condition. This provides a method for finding minimal realizations, as well as allowing us to characterize uniquely …

Class path: 05-XX — Combinatorics › 05Cxx — Graph theory › 05C05 — Trees

## 71. uid `b3b1f07239`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** L(3,2,1)-labelings of three classes of 4-valent circulants

> An $L(3,2,1)$-labeling of a graph $G$ is an assignment $f$ of nonnegative integers to vertices such that $\vert f(x)-f(y)\vert > 3-\mbox{dist}_G(x,y)$ for every pair $x,y$ of vertices of $G$, where $\mbox{dist}_G(x,y)$ denotes the distance between $x$ and $y$ in $G$. The minimum span (i.e., the difference between the largest and the smallest value) among all $L(3,2,1)$-labelings of $G$ is denoted by $\lambda_{(3,2,1)}(G)$. In this paper, we study $L(3,2,1)$-labelings of three classes of circulant graphs. Namely, we investigate $\lambda_{(3,2,1)}$ of circulant graphs $C_n(1,t)$, where $t\in\{3,4,5\}$ and $n$ is the order of the graph. This paper is a continuation of a recent publications of V. Bianco and T. Calamoneri who studied the square of cycles, i.e., circulant graphs $C_n(1,2)$.

Class path: a — forms › an — quantities › ann — units › annq — two › annqU — e

**Work 2.** Thermodynamic Regulation of Superradiance in a Charged Two-Dimensional Black Hole

> Within the framework of gravitational thermodynamicization, we investigate the propagation of charged scalar fields in static two-dimensional black-hole spacetimes. Starting from the scalar-field action, we derive the exact radial wave equation and show that a neutral, massless, minimally coupled scalar field in a genuinely two-dimensional geometry exhibits neither an angular-momentum barrier nor superradiant amplification. For a charged scalar field, the condition for amplification is governed by the electrostatic potential evaluated at the event horizon. In the case of the charged two-dimensional string black hole, the horizon electrostatic potential is directly related to the Hawking temperature, implying that the superradiant frequency window is determined by the thermodynamic state of the black hole. We further formulate a near-horizon residue criterion that provides a …

Class path: a — forms › an — quantities › ann — units › annq — two

## 72. uid `fa6262a5bd`

Scheme: OpenAlex Topics

**Work 1.** An Inexact Riemannian Gradient Descent Algorithm on the Stiefel Manifold with One Newton-Schulz Iteration

> In this paper, we propose an inexact Riemannian gradient descent algorithm on the Stiefel manifold (IRGS-StieONS) using an adaptive step size, where the ``inexact'' refers to the inexactness of retraction. It is proven that one single Newton-Schulz iteration for the retraction is sufficient for global convergence and local linear convergence. Compared to the landing and augmented Lagrangian-based algorithms, the proposed algorithm is the first infeasible algorithm that permits adaptive step sizes with a practical initial step size and guarantees global convergence and local linear convergence under mild assumptions. Moreover, we show that the local convergence rate depends on the condition number of the Riemannian Hessian, which matches the Riemannian steepest descent algorithm. This result implies that the infeasibility in the proposed algorithm does not influence the local convergence …

Class path: D3 — Physical Sciences › F26 — Mathematics › S2612 — Numerical Analysis › T10963 — Advanced Optimization Algorithms Research

**Work 2.** Accelerated Primal-Dual Proximal Gradient Splitting Methods for Convex-Concave Saddle-Point Problems

> In this paper, based a novel primal-dual dynamical model with adaptive scaling parameters and Bregman divergences, we propose new accelerated primal-dual proximal gradient splitting methods for solving bilinear saddle-point problems with optimal nonergodic convergence rates. For the first, using the spectral analysis, we show that a naive extension of acceleration to a quadratic game is unstable. Motivated by this, we present an accelerated primal-dual gradient flow which combines acceleration with careful velocity correction. To work with non-Euclidean distances, we also equip our continuous model with general Bregman divergences and prove the exponential decay of a Lyapunov function. Then, new primal-dual splitting methods are developed based on proper semi-implicit Euler schemes of the continuous model, and the theoretical convergence rates are nonergodic and optimal with respect to …

Class path: D3 — Physical Sciences › F16 — Chemistry › S1607 — Spectroscopy

## 73. uid `4d331baa7f`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** A categorical p-adic Langlands correspondence for GL_2(Q_p)

> Let p at least 5 be prime. We construct a fully faithful functor from the derived category of all smooth p-adic representations of GL_2(Q_p) (with a fixed central character) to a derived category of Ind-coherent sheaves on a stack of (phi,Gamma)-modules.

Class path: e — atoms › ed — period 3 elements › edp — phosphorus › edpa — attributes of #edpa › edpal — P---

**Work 2.** Logarithmic Mixing of Random Walks on Dynamical Random Cluster Models

> We study random walks on dynamically evolving graphs, where the environment is given by a time-dependent subset of the edges of an underlying graph. Concretely, following the recently introduced framework of Lelli and Stauffer, we consider a random walk interacting with a dynamical random-cluster environment, in which edges are updated with rate $\mu>0$ according to Glauber dynamics with parameters $p$ and $q$, and the walker moves at rate 1 but may only traverse edges that are present at the time of the move. This setting introduces strong dependencies between the walk and the environment, as edge-update probabilities depend on the global connectivity structure. We focus on the case where the underlying graph is a random $d$-regular graph and the parameters lie in the subcritical regime $p < p_{\mathrm{u}}(q, d)$ where it is known that the Glauber dynamics mixes quickly. Our main …

Class path: e — atoms › ed — period 3 elements › edp — phosphorus › edpa — attributes of #edpa › edpal — P---

## 74. uid `7ab4417a4e`

Scheme: arXiv category taxonomy

**Work 1.** Imaging of ultrasound specularity at cortical bone interfaces to detect unbalanced remodeling: A preliminary study

> The region near the interface between cortical bone and the medullary cavity (the endosteal surface) is of particular interest for the early detection of osteoporosis. This region is typically the first to exhibit signs of unbalanced remodeling (enlarged pores and increased surface roughness). In this study, we analyze ultrasound reflection at the endosteal surface to introduce a novel potential biomarker of cortical bone health based on specularity. Our hypothesis is that increased pore size and surface roughness enhance ultrasound scattering, thereby reducing specular reflection. We reconstruct maps of specularity by combining a signal processing technique initially designed to enhance specular reflectors in soft tissues with a beamforming technique that accounts for refraction at bone soft tissue interfaces. Specularity values (between 0 and 1) quantify the similarity between …

Class path: grp_physics — Physics › physics — Physics › physics.med-ph — Medical Physics

**Work 2.** Coronal gas excitation as a tracer of supermassive black hole mass: on the mid-IR coronal [Ne v] lines

> Coronal lines (CL) may be a reliable proxy of supermassive black hole (SMBH) masses in active galactic nuclei (AGN) because the production of these high ionisation potential (IP) lines is sensitive to the shape of the ionising continuum which, for a thin accretion disc, relies on the black hole mass. In this work we study the connection between the [Ne\,v] coronal lines and the SMBH mass. We analyse the \textit{Spitzer spectra} of a sample of 27 AGN spanning three orders of magnitude in SMBH mass. Fluxes and the electron density in the coronal gas medium were measured after fitting Gaussians to both the $\rm [Ne\,v]14\mu m$ and $\rm [Ne\,v]24\mu m$ lines where detected at high signal to noise. Line fluxes were normalised to the Br$\gamma_{\rm broad}$ emission from the broad line region. We found strong correlations between the $\rm [Ne\,v]14\mu m/Br\gamma_{\rm broad}$ and $\rm …

Class path: grp_physics — Physics › astro-ph — Astrophysics › astro-ph.GA — Astrophysics of Galaxies

## 75. uid `d617a8e605`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** Error attribution in gradient random walk methods for parabolic equations

> Gradient random walk (GRW) methods represent a solution's spatial derivative by weighted particles and recover the solution by cumulative summation. We examine their accuracy for the one-dimensional heat, scalar cubic reaction-diffusion, and Burgers' equations. Thirty-seed ensembles, paired reconstructions, and deterministic controls identify the sources of two apparent accuracy limits. For the heat equation, comparing cumulative bin sums at bin centers introduces a first-order alignment error, which the exact reflected-particle distribution predicts and bin-edge comparison removes. An identity for the expected squared error predicts the remaining floor and the particle count for a target root-mean-square accuracy. With unequal weights, a count chosen in advance gives an error within 1% of its prediction. Reaction-front position, speed, and shape errors decrease under refinement. For …

Class path: a — forms › an — quantities › ann — units › annp — one

**Work 2.** Discovery of an Unbound Flyby Companion of UBC 63: In the Immediate Aftermath of a Close Encounter

> We re-investigate the open cluster UBC 63 using the Gaia DR3 data and show that, rather than being a single cluster as previously classified, it is a compelling candidate for a double cluster undergoing an unbound flyby interaction. A GMM decomposition performed in the 5D astrometric space reveals the two statistically distinct components, namely UBC 63A (98 members, Age = 21 $\pm$ 4 Myr) and UBC 63B (148 members, Age = 562 $\pm$ 43 Myr). A significant age difference of $\Delta \mathrm{Age} = 541 \pm 43$ Myr between the clusters, rules out coeval formation. Their 3D separation of $60 \pm 29$ pc at the birth-epoch of the younger cluster, indicates that the clusters might have originated from the same molecular cloud complex. At present, the system exhibits a 3D separation of $26 \pm 8$ pc, with a relative velocity of $3.60 \pm 1.80$ km s$^{-1}$. Orbital integrations and \textit{N}-body …

Class path: b — spacetime › bc — space › bcd — solids

## 76. uid `1ec7bf7cee`

Scheme: Free Decimal Correspondence

**Work 1.** Kernelized Advantage Estimation: From Nonparametric Statistics to LLM Reasoning

> Recent advances in large language models (LLMs) have increasingly relied on reinforcement learning (RL) to improve their reasoning capabilities. Three types of approaches have been widely adopted: The first relies on a deep neural network to estimate the value function of the learning policy in order to reduce the variance of the policy gradient. However, estimating and maintaining such a value network incurs substantial computational and memory overhead. The second avoids training a value network by approximating the value function using sample averages. However, it samples a large number of reasoning traces per prompt for accurate value function approximation, making it computationally expensive. The third samples only a single reasoning trajectory per prompt, which reduces computational cost but suffers from poor sample efficiency. This paper focuses on a practical, …

Class path: 300 — Social sciences › 310 — Statistics

**Work 2.** GeoMin: Data-Efficient Semi-Supervised RLVR via Geometric Distribution Modeling

> Reinforcement learning with verifiable rewards (RLVR) significantly advances LLM reasoning, yet it faces a dilemma: standard supervised scaling is throttled by high annotation costs, while unsupervised alternatives suffer from severe model collapse. Recent semi-supervised RLVR methods address this by using a small labeled set to guide unlabeled data, achieving a promising trade-off between training efficacy and annotation cost. However, they suffer from a severe data-efficiency bottleneck due to the reliance on coarse performance heuristics, leaving a vast majority of valuable instances underutilized. To this end, we propose GeoMin, which models global feature distributions on labeled data to decode the structural discrepancy between correct and incorrect rollouts, thereby establishing a robust prior to assess the reliability of self-reward signals and fully unleash the potential of …

Class path: 100 — Philosophy and psychology › 120 — Special metaphysical topics › 126 — Self (Philosophy); Consciousness

## 77. uid `3ec3124242`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** Dual-guided Hierarchical Edge Localization for Large-scale Optimal Transport Across Dimensions

> Optimal transport (OT) compares distributions and aligns datasets in machine learning, yet unregularized discrete OT requires a linear program with quadratically many transport variables. We propose HELLO, a hierarchical solver that casts large-scale discrete OT as edge localization and uses dual potentials to guide both coarse-to-fine initialization and within-level refinement. Initialization propagates coarse dual potentials across a recursive subsampling hierarchy to assign candidate edges. Refinement then iteratively inserts the largest dual violators in each row and column until the relative KKT residual meets a prescribed tolerance, while budgeted pruning ensures linear memory complexity. For exact-arithmetic refinement, we prove finite termination at a global optimum under a symbolic lexicographic rule. At the million-point scale, HELLO attains lower transport objectives with …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.01 — Applied mathematics, analysis and probability › 02.01.01.01 — Convex and nonconvex optimization

**Work 2.** Ray-traced weak lensing convergence in screened modified gravity theories

> Weak gravitational lensing is one of the primary cosmological probes, providing powerful constraints on the cosmological model. As Stage IV surveys are expected to deliver data of unprecedented precision, accurate modeling of weak gravitational lensing observables across both linear and non-linear scales becomes increasingly important. In this work, we investigate weak lensing in modified gravity (MG) models, extensions of the standard $\Lambda$CDM cosmology in which gravity deviates from general relativity, generally introducing modifications to the lensing equation. We parametrize these modifications through the common phenomenological function $\Sigma_\mathrm{mg}$ and apply ray-tracing to the density maps of N-body and hydrodynamical simulations. We model the time dependence of $\Sigma_\mathrm{mg}$ analytically, while we introduce a phenomenological scale dependence to represent the …

Class path: 02 — Mathematics and physical sciences › 02.02 — Physics › 02.02.04 — Gravitation and theoretical physics › 02.02.04.01 — Cosmology and the early universe

## 78. uid `89bb93a76a`

Scheme: Free Decimal Correspondence

**Work 1.** Optimal Control of Sweeping Processes with Nonsmooth Moving Sets: A Numerical Algorithm

> This paper extends the numerical method introduced by de Pinho et al. [22] and later developed in Nour and Zeidan [33, 37] to the nonautonomous case where the nonsmooth sweeping set depends explicitly on time. The moving character of the sweeping set creates substantial additional difficulties, since several geometric constants involved in the analysis may a priori depend on time. To overcome this issue, we establish uniform geometric estimates for the moving constraint sets and their boundaries. These estimates allow us to extend the discrete approximation scheme developed in [22, 33, 37] and prove its convergence toward admissible solutions of the original problem. Several examples illustrating the applicability of the proposed method are also presented.

Class path: 600 — Industrial arts; Technology; Engineering › 670 — Manufactures › 676 — Paper; Paper products

**Work 2.** Probability Hacking and the Design of Trustworthy ML for Signal Processing in C-UAS: A Scenario Based Method

> In order to counter the various threats manifested by Unmanned Aircraft Systems (UAS) adequately, specialized Counter Unmanned Aircraft Systems (C-UAS) are required. Enhancing C-UAS with Emerging and Disruptive Technologies (EDTs) such as Artificial Intelligence (AI) can lead to more effective countermeasures. In this paper a scenario-based method is applied to C-UAS augmented with Machine Learning (ML), a subset of AI, that can enhance signal processing capabilities. Via the scenarios-based method we frame in this paper probability hacking as a challenge and identify requirements which can be implemented in existing Rule of Law mechanisms to prevent probability hacking. These requirements strengthen the trustworthiness of the C-UAS, which feed into justified trust - a key to successful Human-Autonomy Teaming, in civil and military contexts. Index Terms: C-UAS, Scenario-based method, …

Class path: 600 — Industrial arts; Technology; Engineering › 670 — Manufactures › 676 — Paper; Paper products

## 79. uid `3f59b5365b`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Proximity-CLIP: Text-Guided Semantic Proximity Learning for Zero-Shot Anomaly Detection

> Vision-language models offer a promising approach for zero-shot anomaly detection (ZSAD). However, due to object-centric bias, normal and anomalous text prototypes exhibit a high semantic overlap. While enforcing strict orthogonality between them improves discriminability, mapping highly contiguous visual inputs onto drastically orthogonal prototypes introduces a geometric dilemma, disrupting the pre-trained structural continuity. To address this problem, we propose Proximity-CLIP, a framework that visually calibrates the semantic margin to guide visual adaptation. First, we introduce a visually-calibrated semantic proximity learning mechanism that uses a bounded dynamic regularization to learn an appropriate semantic margin, ensuring discriminative separation while preserving structural alignment. Second, we design an Anomaly Query Module (AQM) driven by these text priors. Using the …

Class path: P — Language and literature › PQ — French literature - Italian literature - Spanish literature - Portuguese literature › PQ6001-8929 — Spanish literature › PQ6001-6168 — History and criticism › PQ6075-6098 — Poetry › PQ6097.A-6097.Z — Other, A-Z › PQ6097.V57 — Visual

**Work 2.** Integration of deep generative Anomaly Detection algorithm in high-speed industrial line

> Industrial visual inspection in pharmaceutical production requires high accuracy under strict constraints on cycle time, hardware footprint, and operational cost. Manual inline inspection is still common, but it is affected by operator variability and limited throughput. Classical rule-based computer vision pipelines are often rigid and difficult to scale to highly variable production scenarios. To address these limitations, we present a semi-supervised anomaly detection framework based on a generative adversarial architecture with a residual autoencoder and a dense bottleneck, specifically designed for online deployment on a high-speed Blow-Fill-Seal (BFS) line. The model is trained only on nominal samples and detects anomalies through reconstruction residuals, providing both classification and spatial localization via heatmaps. The training set contains 2,815,200 grayscale patches. …

Class path: H — Social sciences › HT — Communities. Classes. Races › HT601-1445 — Classes › HT675-690 — Classes arising from occupation › HT685 — Industrial

## 80. uid `fb8164a02c`

Scheme: OpenAlex Topics

**Work 1.** Solvable compatible Lie algebras with a given nilradical

> We extend the classical construction of solvable Lie algebras from a nilradical to compatible Lie algebras. Since the sum of nilpotent ideals may fail to be nilpotent, we replace the usual nilradical by a \emph{special nilradical} that behaves well with the mixed Jacobi identity. We use the maximal tori of diagonal derivations to build solvable extensions. The method is applied to the pairs $(\mathrm L_n,\mathrm R_n)$ and $(\mathrm L_n,\mathrm W_n)$, yielding explicit one-dimensional solvable extensions and proving nonexistence of higher-dimensional ones in these cases. We also study filiform compatible Lie algebras. We introduce the model family $\mathcal L_s$ and show that each $\mathcal L_s$ is a linear deformation of the model filiform Lie algebra $\mathcal L_k$. Finally, we study the existence of solvable extensions of this family, within the framework developed above.

Class path: D3 — Physical Sciences › F26 — Mathematics › S2602 — Algebra and Number Theory › T11673 — Advanced Topics in Algebra

**Work 2.** Weighted Null Space Fitting (WNSF): A Link between The Prediction Error Method and Subspace Identification

> Subspace identification methods (SIMs) have proven to be very useful and numerically robust for building state-space models. While most SIMs are consistent, few if any can achieve the efficiency of the maximum likelihood estimate (MLE). Conversely, the prediction error method (PEM) with a quadratic criteria is equivalent to MLE, but it comes with non-convex optimization problems and requires good initialization points. This contribution proposes a weighted null space fitting (WNSF) approach for estimating state-space models, combining some key advantages of the two aforementioned mainstream approaches. It starts with a least-squares estimate of a high-order ARX model, and then a multi-step least-squares procedure reduces the model to a state-space model on canoncial form. It is demonstrated through statistical analysis that when a canonical parameterization is admissible, the proposed …

Class path: D3 — Physical Sciences › F22 — Engineering › S2207 — Control and Systems Engineering › T11236 — Control Systems and Identification

## 81. uid `c5de08f589`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Quasi-pointed exactness and torsion

> We show that, under some mild assumptions, every quasi-pointed category C carries a canonical near-torsion theory (T,F), where T consists of the 0-supported objects, that is, those admitting a morphism to the initial object 0, and F consists of the subterminal objects. The coreflection and the reflection of an object A are then given by the projection $A \times 0 \to A$ and by the strong-epimorphic part of $A \to 1$, respectively. Using the general construction of the "largest" non-pointed torsion theory associated with a near-torsion theory, we prove that (T,F) is a (non-pointed) torsion theory in the full subcategory of those objects A for which the canonical sequence $t(A) \to A \to f(A)$ is short exact, in the sense that it is both a kernel and a cokernel diagram. This subcategory is the whole of C whenever C is sequentiable, that is, a regular protomodular and quasi-pointed …

Class path: 18-XX — Category theory; homological algebra › 18Exx — Categorical algebra › 18E40 — Torsion theories, radicals

**Work 2.** Fishing for complements

> Given a presilting object in a triangulated category, we find necessary and sufficient conditions for the existence of a complement. This is done both for classic (pre)silting objects and for large (pre)silting objects. The key technique is the study of associated co-t-structures. As a consequence of our techniques we recover some known cases of the existence of complements, including for derived categories of some hereditary abelian categories and for silting-discrete algebras. Moreover, we also show that a finite-dimensional algebra is silting discrete if and only if every bounded large silting complex is equivalent to a compact one.

Class path: 18-XX — Category theory; homological algebra › 18Gxx — Homological algebra in category theory, derived categories and functors › 18G05 — Projectives and injectives (category-theoretic aspects)

## 82. uid `0cd5988e22`

Scheme: UNESCO Thesaurus

**Work 1.** On the category of semi-graded modules

> Lezama \cite{LezamaLatorre2017} introduced the notion of semi-graded ring with the aim of generalizing $\mathbb{Z}$-graded rings and several families of noncommutative rings of polynomial type non-$\mathbb{N}$-graded such as the skew Poincar\'e-Birkhoff-Witt extensions defined by him \cite{GallegoLezama2010}. In a series of papers, \cite{Lezama2020, Lezama2021, LezamaGomez2019, LezamaLatorre2017}, he studied problems of non-commutative projective algebraic geometry generalizing the original ideas of Artin et al. \cite{Artin1992, ArtinSchelter1987, ArtinTateVandenBergh2007, ArtinTateVandenBergh1991, ArtinZhang1994} on $\mathbb{N}$-graded rings, in the categorical context of the category $\mathsf{SGR}-R$ of left semi-graded modules over a semi-graded ring $R$. In this note we prove that $\mathsf{SGR}-R$ possesses a canonical set of free generators via shifted twists, which endows the …

Class path: 2 — Science › 2.15 — Mathematics and statistics › c118 — Mathematics › c11451 — Geometry

**Work 2.** Asymptotic Analysis of the Narrow Escape and Berg-Purcell problems on general three-dimensional domains with reactive boundary patches

> We present an asymptotic analysis of two diffusive capture problems in general smooth closed three-dimensional geometries with multiple small reactive boundary patches of arbitrary shapes. (i) The narrow escape problem seeks to determine the escape rate of Brownian particles from an enclosed region through small boundary windows. (ii) The related Berg-Purcell (or narrow entrance) problem seeks to resolve the capture rate for signaling molecules diffusing outside the cell and entering through localized reactions at membrane-bound receptors. We obtain matched asymptotic solutions of these two problems and thus address the long-standing challenge of describing the role that curvature and local reactivities play in modulating diffusive capture rates. Our explicit expansions quantify local effects on diffusive capture through the sizes, shapes, and reactivities of the patches together with …

Class path: 2 — Science › 2.15 — Mathematics and statistics › c118 — Mathematics › c11451 — Geometry

## 83. uid `a35ded2c7a`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** The largest Laplacian eigenvalue of induced-$K_{1,r}$-free graphs

> Let $G$ be a simple graph of maximum degree $d$, and let $\mu(G)$ denote the largest eigenvalue of its Laplacian matrix. For a fixed integer $k\geq 2$, Aharoni, Alon, and Berger (2016) asked whether every graph containing no induced copy of $K_{1,k}$ satisfies $\mu(G)\leq (2 - \frac{2}{k} + o(1)) d$. We answer this question by proving the stronger sharp bound \[ \mu(G)\leq \left(2-\frac{2}{k}\right)(d+1). \] The proof combines a sign decomposition of a Laplacian Rayleigh vector with a weighted local Caro-Wei type inequality for independent sets.

Class path: e — atoms › ee — period 4 elements › eeb — potassium › eeba — attributes of #eeba › eebap — K+

**Work 2.** When positive and negative pairs differ in femtoscopy: residual Coulomb and isospin effects

> We study charge-dependent modifications of identical-pion and identical-kaon femtoscopic correlation functions from two sources: the residual Coulomb field of the charged source and isospin-related hadronic dynamics. The residual Coulomb effect is modeled with a modified Retiere--Lisa blast-wave source, where the same emitted particles are propagated with positive and negative charge signs through an effective residual field. The residual Coulomb field produces a small but systematic positive-to-negative splitting of the correlation functions, strongest at low $k_T$ and sensitive to the effective charge, spatial width, and expansion velocity of the residual source. It also modifies the height and shape of the correlation function, complicating interpretation of the fitted radii. UrQMD 3.4 cascade calculations for Au+Au collisions at $\sqrt{s_{NN}}=7.7~\mathrm{GeV}$ show that …

Class path: e — atoms › ee — period 4 elements › eeb — potassium › eeba — attributes of #eeba › eebap — K+

## 84. uid `a2d4fe1d41`

Scheme: OpenAlex Topics

**Work 1.** AD$^2$: Analysis and Detection of Adversarial Threats in Visual Perception for End-to-End Autonomous Driving Systems

> End-to-end autonomous driving systems have achieved significant progress, yet their adversarial robustness remains largely underexplored. In this work, we conduct a closed-loop evaluation of state-of-the-art autonomous driving agents under black-box adversarial threat models in CARLA. Specifically, we consider three representative attack vectors on the visual perception pipeline: (i) a physics-based blur attack induced by acoustic waves, (ii) an electromagnetic interference attack that distorts captured images, and (iii) a digital attack that adds ghost objects as carefully crafted bounded perturbations on images. Our experiments on two advanced agents, Transfuser and Interfuser, reveal severe vulnerabilities to such attacks, with driving scores dropping by up to 99% in the worst case, raising valid safety concerns. To help mitigate such threats, we further propose a lightweight Attack …

Class path: D3 — Physical Sciences › F22 — Engineering › S2203 — Automotive Engineering › T11099 — Autonomous Vehicle Technology and Safety

**Work 2.** NIC-RobustBench: A Comprehensive Open-Source Toolkit for Neural Image Compression and Robustness Analysis

> Neural image compression (NIC) is increasingly used in computer vision pipelines, as learning-based models are able to surpass traditional algorithms in compression efficiency. However, learned codecs can be unstable and vulnerable to adversarial attacks: small perturbations may cause severe reconstruction artifacts or indirectly break downstream models. Despite these risks, most NIC benchmarks only emphasize rate-distortion (RD) performance, focusing on model efficiency in safe, non-adversarial scenarios, while NIC robustness studies cover only specific codecs and attacks. To fill this gap, we introduce \textbf{NIC-RobustBench}, an open-source benchmark and evaluation framework for adversarial robustness of NIC methods. The benchmark integrates 8 attacks, 9 defense strategies, standard RD metrics, a large and extensible set of codecs, and tools for assessing both the robustness of the …

Class path: D3 — Physical Sciences › F17 — Computer Science › S1702 — Artificial Intelligence › T11689 — Adversarial Robustness in Machine Learning

## 85. uid `e42ace2683`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** CREX and PREX-II reconciled within energy-density functional theory

> The CREX and PREX-II measurements of the neutron-skin thickness of 48Ca and 208Pb challenge standard nuclear energy-density functional (EDF) descriptions of nuclei and nuclear matter. We show that the apparent tension arises from an implicit constraint in EDF theory, which ties the density dependence of the functional at the dilute nuclear surface to that of uniform matter near saturation. Relaxing this surface-bulk coupling and independently constraining the dilute-density sector, while preserving realistic saturation and high-density behavior, yields EDFs that simultaneously reproduce the neutron skins of both nuclei, their electric dipole polarizabilities, and neutron-star mass-radius relations. The established correlation between the neutron-skin thickness and the symmetry-energy slope parameter L at saturation is retained but becomes substantially broader. The results show that …

Class path: Q — Science › QC — Physics › QC501-766 — Electricity and magnetism › QC501-721 — Electricity › QC717.6-718.8 — Plasma physics. Ionized gases › QC718.5.A-718.5.Z — Special topics, A-Z › QC718.5.D4 — Density

**Work 2.** Design Studies Of A Pulsed Quasimonoenergetic 2-keV Neutron Source For Calibration Of Low Threshold Dark Matter Detectors

> We describe design studies for a pulsed quasi-monoenergetic 2-keV neutron source for calibration of sub-keV nuclear recoils. Such a calibration is required for detectors sensitive to sub- GeV dark matter and also the coherent elastic scattering of reactor neutrinos. In our design, neutrons from a commercial deuterium-tritium generator are moderated to the keV scale and subsequently filtered to achieve a monoenergetic spectrum utilizing the antiresonance feature, characterized by a drop in neutron transmission spectra at 2-keV for scandium. The analyses and conclusions presented are primarily based on simulations, as this study represents a critical first step toward the final goal. We address challenges such as unmoderated high-energy neutrons and gamma backgrounds from neutron capture in the moderator materials. Through optimization of the moderator+filter and shielding geometry, we …

Class path: U — Military science › UG — Military engineering. Air forces › UG622-1435 — Air forces. Air warfare › UG1100-1435 — Equipment and supplies › UG1282.A-1282.Z — By type, A-Z › UG1282.N48 — Neutron

## 86. uid `53de4a941a`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** The effect of geometric focusing on dispersive estimates for the Schr\"odinger and wave equations

> We classify the long-time decay rate in dispersive estimates for the Schr\"odinger and wave equations on non-trapping asymptotically conic manifolds and exact metric cones in terms of the intensity of geometric focusing. Letting $X_0$ be a metric cone, one of our main results demonstrates that each multiplicity of conjugate points within distance $\pi$ on $Y=\partial X_0$ leads to a $|t|^{1/2}$-loss in the long-time decay order and a half-order shift in the regularity index in the dispersive estimate for the Schr\"odinger equation. Unexpectedly, conjugate point pairs on $Y$ at distance $\pi$ do not cause loss when the Legendre submanifold carrying the wave propagation satisfies a natural admissible condition that we propose. In sum, we give a robust framework for proving dispersive estimates that is stable under geometric perturbations and also accommodates perturbations by potentials.

Class path: 35-XX — Partial differential equations › 35Qxx — Partial differential equations of mathematical physics and other areas of application › 35Q41 — Time-dependent Schrödinger equations and Dirac equations

**Work 2.** IsUMap: Manifold Learning and Data Visualization leveraging Vietoris-Rips filtrations

> This work introduces IsUMap, a novel manifold learning technique that enhances data representation by integrating aspects of UMAP and Isomap with Vietoris-Rips filtrations. We present a systematic and detailed construction of a metric representation for locally distorted metric spaces that captures complex data structures more accurately than the previous schemes. Our approach addresses limitations in existing methods by accommodating non-uniform data distributions and intricate local geometries. We validate its performance through extensive experiments on examples of various geometric objects and benchmark real-world datasets, demonstrating significant improvements in representation quality.

Class path: 51-XX — Geometry › 51Kxx — Distance geometry › 51K05 — General theory of distance geometry

## 87. uid `9aa6cc8876`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** A globally convergent filter-trust-region method for large deformation contact problems

> We present a globally convergent method for the solution of frictionless large deformation contact problems for hyperelastic materials. The discretisation uses the mortar method which is known to be more stable than node-to-segment approaches. The resulting non-convex constrained minimisation problems are solved using a filter-trust-region scheme, and we prove global convergence towards first-order optimal points. The constrained Newton problems are solved robustly and efficiently using a Truncated Non-smooth Newton Multigrid (TNNMG) method with a Monotone Multigrid (MMG) linear correction step. For this we introduce a cheap basis transformation that decouples the contact constraints. Numerical experiments confirm the stability and efficiency of our approach.

Class path: 65-XX — Numerical analysis › 65Kxx — Numerical methods for mathematical programming, optimization and variational techniques › 65K15 — Numerical methods for variational inequalities and related problems

**Work 2.** Markov and lattice bases for Forman-Ricci curvature of graphs

> Discrete Forman-Ricci curvature is a quantity associated to each edge of a graph that describes its local geometry. It has proven to be a useful tool in network analysis in a variety of applications. Recent work by Roost et al.\ (2024) proposed the use of Markov bases to sample from the space of graphs with prescribed vertex degrees and curvatures. In the present work, we further develop the algebraic and combinatorial theory of these Markov bases. We show that the degree of an indispensable Markov move grows at least quadratically in the maximum degree of the graph. In light of this result, a compact description of all Markov basis elements seems unattainable at present. Instead, we find a lattice basis for this problem using only degree three Markov moves, which allows us to employ recently-developed reinforcement learning methods for finding Markov moves that can be applied to a …

Class path: 05-XX — Combinatorics › 05Cxx — Graph theory › 05C10 — Planar graphs; geometric and topological aspects of graph theory

## 88. uid `f106514365`

Scheme: OpenAlex Topics

**Work 1.** Information-geometric adaptive sampling for graph diffusion

> Standard diffusion models for graph generation typically rely on uniform time-stepping, an approach that overlooks the non-homogeneous dynamics of distributional evolution on complex manifolds. In this paper, we present an information-geometric framework that reinterprets the diffusion sampling trajectory as a parametric curve on a Riemannian manifold. Our key observation is that the Fisher-Rao metric provides a principled measure of the intrinsic distance. By analyzing this metric, we derive the Drift Variation Score (DVS), a geometry-aware indicator that quantifies the instantaneous rate of distributional change. Unlike prior heuristic-based adaptive samplers, our DVS solver enforces a constant informational speed on the statistical manifold, automatically maintaining a uniform rate of distributional change along the sampling trajectory. This equal arc-length strategy ensures that …

Class path: D3 — Physical Sciences › F17 — Computer Science › S1705 — Computer Networks and Communications

**Work 2.** Constructing the Field of Philanthropic and Nonprofit Studies: Evidence from Citation Networks

> Philanthropic and nonprofit studies (PNPS) has grown rapidly as an interdisciplinary field, yet its intellectual structure and boundaries remain only partially visible. This study maps the field by combining journal- and keyword-based retrieval with citation network analysis, natural language processing, and large language model-assisted cluster labeling. Using 60,917 Web of Science articles, we identify major topical communities, examine their structural connections, and assess how well mainstream PNPS journals represent the broader landscape of related scholarship. The results show that PNPS is a loosely connected field dominated by three distinct "power centers": nonprofit organizations, social movements, and voluntary action. The analysis also reveals a broader disciplinary footprint than commonly recognized, extending beyond the social sciences and humanities into areas such as …

Class path: D3 — Physical Sciences › F17 — Computer Science › S1705 — Computer Networks and Communications

## 89. uid `090689cca4`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** Bridge Routing Heads: Where Multilingual Multi-hop Reasoning Lives in LLMs

> Multilingual LLMs answer the same multi-hop reasoning question across languages, but we lack a mechanistic account of whether they share an internal circuit. We identify Bridge Routing Heads (BRH) in two large multilingual LLMs through a three-stage pipeline. The resulting language-specific head sets exhibit near-complete mutual exclusivity across the five languages, with a mean Jaccard similarity of only 0.017 for Llama 3.1 70B and 0.057 for Qwen 2.5 72B, revealing language-idiosyncratic circuits. Ablating general BRH increases two-hop Negative Log-Likelihood (NLL) by 39-89x the random-head baseline, providing direct causal evidence of their role. Amplifying these heads in a failing target-language pass rescues up to 51.7% of cross-lingual failures, with no training. The two models share this dual-circuit pattern but allocate heads differently: Llama concentrates chaining in a large …

Class path: 01 — Computing, AI and engineering

**Work 2.** Amortized Reasoning Tree Search: Decoupling Proposal and Decision in Large Language Models

> Reinforcement Learning with Verifiable Rewards (RLVR) has established itself as the dominant paradigm for instilling rigorous reasoning capabilities in Large Language Models. While effective at amplifying dominant behaviors, we identify a critical pathology in this alignment process: the systematic suppression of valid but rare (low-likelihood under the base model distribution) reasoning paths. We theoretically characterize this phenomenon as a "Normalization Squeeze," where the interplay between mode-seeking policy gradients and finite sampling acts as a high-pass likelihood filter, driving the probability of rare correct traces to statistical extinction. To counteract this collapse without discarding the base model's latent diversity, we propose Amortized Reasoning Tree Search (ARTS). Unlike standard approaches that force internalization via parameter updates, ARTS prioritizes …

Class path: 01 — Computing, AI and engineering

## 90. uid `0fefa0ea91`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** An Object-Oriented Spatial Statistics Approach for Human Activity Space Estimation

> Human activity spaces are shaped by individual mobility and the built environment, motivating statistical methods that integrate GPS observations with GIS representations of places and routes. We propose a novel methodology to estimate activity spaces in built environments from GPS data within the Object Oriented Spatial Statistics framework. We characterize daily mobility through the distribution of time across spatial polygons and road segments, aiming to capture entity-specific time-use fractions and level-$\gamma$ activity spaces. We develop a time-weighted estimator to handle irregularly sampled GPS observations. We derive an error bound that quantifies the effects of measurement error, nearest-entity misclassification, temporal gaps, boundary crossings, and day-to-day variability. We also develop a map-augmented representation of daily activity patterns, a dwell-time-weighted …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.02 — Pure mathematics › 02.01.02.02 — Functional analysis and operator theory

**Work 2.** A Unified Range Characterization for the Spherical mean transform

> In this article, we investigate the range characterization for the spherical mean transform (SMT) of functions supported in the unit ball. In earlier works, in the case of odd dimensions, a set of differential conditions was obtained, whereas in the case of even dimensions, integral conditions were obtained. We prove that these conditions that are different based on the parity of dimension are, in fact, equivalent in odd dimensions. This equivalence shows that the integral conditions yield a unified simple range characterization for the SMT that is valid in both even and odd dimensions.

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics

## 91. uid `4229eecaeb`

Scheme: Free Decimal Correspondence

**Work 1.** Blow-up or grow-up for the focusing 3D cubic NLS with a repulsive inverse-power potential at the mass--energy threshold

> We consider the focusing cubic nonlinear Schrodinger equation with a repulsive inverse-power potential $V(x)=a|x|^{-\mu}$, where $a>0$ and $1<\mu\leq 2$. At the mass-energy threshold, Miao, Murphy, and Zheng, as well as Ardila, Hamano, and Ikeda, established sharp scattering results in the positive virial region. In this paper, we continue to study the dynamics in the negative virial region and show that, in each time direction, solutions either blow up in finite time or grow up.

Class path: 600 — Industrial arts; Technology; Engineering › 670 — Manufactures › 676 — Paper; Paper products

**Work 2.** A Comparison of some Weighted Numerical Radii of Hilbert Space Operators

> In recent years, several generalizations of the numerical radius for bounded linear operators on Hilbert spaces have been introduced and extensively studied. These generalizations provide refined tools for investigating operator inequalities and spectral properties. In this paper, we investigate several generalized numerical radii and establish relationships among them. We also establish several relationships among these generalized numerical radii and show that the $(s,t)$-weighted numerical radius can be represented as a rescaled form of the $t$-weighted numerical radius.

Class path: 600 — Industrial arts; Technology; Engineering › 670 — Manufactures › 676 — Paper; Paper products

## 92. uid `d75ff7a453`

Scheme: UNESCO Thesaurus

**Work 1.** Single-Pulse Study of the Pseudo-nulling Pulsar PSR J1820-0509 Based on FAST Observations

> Using two observations obtained with the Five-hundred-meter Aperture Spherical radio Telescope (FAST), we present a detailed single-pulse analysis of the high-nulling pulsar PSR J1820-0509. We measure an exceptionally high nulling fraction of approximately 81.78%, significantly exceeding previous estimates from Parkes observations. The single-pulse energy distribution exhibits a clear bimodal structure, consistent with classical nulling behavior. However, stacking the identified null pulses reveals a statistically significant residual profile above the noise level, indicating that the nulls correspond to a very weak emission state rather than a complete cessation of radio emission. The pulsar shows clustered burst activities spanning several hundred rotation periods, with prominent quasi-periodicities at 1191 +/- 81 and 590 +/- 15 pulse periods in the two observations. Based on temporal …

Class path: 2 — Science › 2.65 — Natural resources › c212 — Energy resources

**Work 2.** Reliability of Dynamic Load Scheduling with Solar Forecast Scenarios

> This paper presents and evaluates the performance of an optimal scheduling algorithm that selects the on/off combinations and timing of a finite set of dynamic electric loads on the basis of short term predictions of the power delivery from a photovoltaic source. In the algorithm for optimal scheduling, each load is modeled with a dynamic power profile that may be different for on and off switching. Optimal scheduling is achieved by the evaluation of a user-specified criterion function with possible power constraints. The scheduling algorithm exploits the use of a moving finite time horizon and the resulting finite number of scheduling combinations to achieve real-time computation of the optimal timing and switching of loads. The moving time horizon in the proposed optimal scheduling algorithm provides an opportunity to use short term (time moving) predictions of solar power based on …

Class path: 2 — Science › 2.65 — Natural resources › c212 — Energy resources › c9750 — Renewable energy sources › c9719 — Solar energy

## 93. uid `ecb01df65f`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Utility Maximisation with Model-independent Constraints

> We consider an agent who has access to a financial market, including derivative contracts, who looks to maximise her utility. Whilst the agent looks to maximise utility over one probability measure, or class of probability measures, she must also ensure that the mark-to-market value of her portfolio remains above a given threshold. When the mark-to-market value is based on a more pessimistic valuation method, such as model-independent bounds, we recover a novel optimisation problem for the agent where the agents investment problem must satisfy a pathwise constraint. For complete markets, the expression of the optimal terminal wealth is given, using the max-plus decomposition for supermartingales. Moreover, for the Black-Scholes-Merton model the explicit form of the process involved in such decomposition is obtained, and we are able to investigate numerically optimal portfolios in the …

Class path: P — Language and literature › PR — English literature › PR1803-2165 — Anglo-Norman period. Early English. Middle › PR1875.A-1875.Z — Special topics, A-Z › PR1875.V34 — Value

**Work 2.** Mean Field Portfolio Games with Epstein-Zin Preferences

> We study mean field portfolio games under Epstein-Zin preferences, which naturally encompass the classical time-additive power utility as a special case. In a general non-Markovian framework, we establish a uniqueness result by proving a one-to-one correspondence between Nash equilibria and the solutions to a class of BSDEs. A key ingredient in our approach is a necessary local stochastic maximum principle, applied to log-wealth, tailored to Epstein-Zin utility, and a nonlinear transformation. In the deterministic setting, we further derive an explicit closed-form solution for equilibrium investment and consumption policies. The strength of our approach is further illustrated by two special cases: (i) in the power utility setting without consumption, we obtain the same one-to-one correspondence as in Fu and Zhou [22] under exactly the same assumption, but without invoking the dynamic …

Class path: G — Geography. Anthropology. Recreation › GV — Recreation. Leisure › GV1199-1570 — Games and amusements › GV1221-1469.63 — Indoor games and amusements › GV1312-1469 — Board games. Move games › GV1469.15-1469.62 — Computer games. Video games. Fantasy games › GV1469.35.A-1469.35.Z — Individual games, equipment, etc., A-Z › GV1469.35.O53 — One

## 94. uid `f6bf68f221`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** Frustrated out-of-plane Dzyaloshinskii-Moriya interaction and the onset of atomic-scale 3$q$ magnetic textures in 2D Fe$_{3}$GeXTe (X = Te, Se, S) monolayers

> We theoretically study the effect of in- and out-of-plane Dzyaloshinskii-Moriya interaction (DMI) on the magnetic ground states of two-dimensional (2D) Fe$_3$GeXTe (X=Te, Se, S) monolayers, where X=Se, S correspond to antisymmetric Janus structures with nonvanishing in-plane DMI. We perform atomistic spin simulations with the extended Heisenberg Hamiltonian parametrized by first principles calculations. While we find that the base DMI in all systems is too weak to stabilize noncollinear states, we show how the frustrated out-of-plane DMI tends to favor atomic-scale $3q$ magnetic textures at the edge of the Brillouin zone. Owing to the ability to tune the DMI in 2D magnets via applied strain or electric field, we study the evolution of the systems' ground state with increasing DMI amplitude. We find that nonplanar $3q$ states are favored under scaling factors as low as 3, while larger …

Class path: e — atoms › ed — period 3 elements › edq — sulfur › edqa — attributes of #edqa › edqam — S--

**Work 2.** SurgicalMamba: Dual-Path SSD with State Regramming for Online Surgical Phase Recognition

> Online surgical phase recognition must commit to a prediction at every frame of a procedure that runs for hours, from past frames alone and at a per-frame cost that does not grow with elapsed length. Structured state-space duality (SSD) meets that constraint, but only by having the scan see a per-head scalar transition, which fixes both where the state puts a frame and how fast it decays. The same views recur through an operation, so repeated content is written over itself and can afterwards be told apart only by age. How fast to decay is left to the step, and when the past stops being useful has to be inferred from a loss that never marks the moment. Procedures run long and change little visually from frame to frame, leaving the step with little to select on. Phases also vary widely in length, so no fixed rate serves as a fallback. We address the two with two mechanisms. State …

Class path: e — atoms › ed — period 3 elements › edq — sulfur › edqa — attributes of #edqa › edqam — S--

## 95. uid `0fc45acbbf`

Scheme: UNESCO Thesaurus

**Work 1.** Pulling strings in real time: flux tube dynamics in (2+1)-d $\mathbb{Z}_2$-Higgs Gauge Theories

> Understanding real-time flux-tube dynamics in more than one spatial dimension is key to unlocking the non-perturbative physics of confinement, and is now actively pursued by quantum computing and simulation experiments. However, describing such dynamics has proven to be extremely challenging with both experiments and state-of-the-art numerical simulations limited to small volumes and short timescales. Here we investigate flux tube statics and real-time evolution in a genuine two-dimensional $\mathbb{Z}_2$ Higgs gauge theory at system sizes and timescales order of magnitude beyond present experiments and numerics. The key enabling element is the recently introduced Clifford-augmented matrix product states (CAMPS) framework, which we demonstrate to parametrically reduce the entanglement that must be represented in the matrix product state; both in the pure-gauge limit and in the presence …

Class path: 2 — Science › 2.30 — Space sciences › c155 — Time

**Work 2.** Proposal to use laser-accelerated electrons to probe the axion-electron coupling

> The axion is a hypothetical particle associated with a possible solution to the strong CP problem and is a leading candidate for dark matter. In this paper we investigate the emission of axions by accelerated electrons. We find the emission probability and energy within the WKB approximation for an electron accelerated by an electromagnetic field. As an application, we estimate the number of axions produced by electrons accelerated using two counter-propagating high-intensity lasers and discuss how they would be converted to photons to be detected. We find that, under realistic experimental conditions, competitive model-independent bounds on the coupling between the axion and the electron could be achieved in such an experiment.

Class path: 2 — Science › 2.25 — Chemical sciences › c144 — Elementary particles › c9543 — Electrons

## 96. uid `99ddef9ac7`

Scheme: UNESCO Thesaurus

**Work 1.** When Prompts Ignore Structure: Graph-Based Attribute Reasoning for Calibrated VLMs

> Reliable confidence estimation remains a key limitation of test-time adaptation in vision-language models (VLMs), where prompt tuning improves zero-shot accuracy but often degrades calibration due to entropy-driven overconfidence. Prior approaches mitigate this using LLM-derived class attributes and contrastive regularization, yet treat attributes independently, ignoring their relational structure. We propose ARGTCA, which represents (class, attribute) pairs as nodes in a Symbolic Attribute Graph and trains a Graph Attention Network (GAT) using contrastive objectives to produce structurally informed embeddings that capture inter-attribute dependencies. We introduce two attribute selection strategies: ARGTCA-DIV for intra-class diversity and ARGTCA-DISC for inter-class discrimination. Experiments across nine benchmarks show that ARGTCA-DIV reduces average Expected Calibration Error (ECE) …

Class path: 2 — Science › 2.30 — Space sciences › c155 — Time

**Work 2.** Seeing Right but Saying Wrong: Inter- and Intra-Layer Refinement in MLLMs without Training

> Multimodal Large Language Models (MLLMs) have demonstrated strong capabilities across a variety of vision-language tasks. However, their internal reasoning often exhibits a critical inconsistency: although deeper layers may attend to the correct visual regions, final predictions are frequently misled by noisy attention from earlier layers. This results in a disconnect between what the model internally understands and what it ultimately expresses, a phenomenon we describe as seeing it right but saying it wrong. To address this issue, we propose DualPD, a dual-perspective decoding refinement strategy that enhances the visual understanding without any additional training. DualPD consists of two components. (1) The layer-wise attention-guided contrastive logits module captures how the belief in the correct answer evolves by comparing output logits between layers that exhibit the largest …

Class path: 1 — Education › 1.05 — Educational sciences and environment › c5 — Learning › c1514 — Learning processes › c3253 — Attention

## 97. uid `50162cb9b6`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Rule Extraction in Machine Learning: Chat Incremental Pattern Constructor

> Rule extraction is a central problem in interpretable machine learning because it seeks to convert opaque predictive behavior into human-readable symbolic structure. This paper presents Chat Incremental Pattern Constructor (ChatIPC), a lightweight incremental symbolic learning system that extracts ordered token-transition rules from text, enriches them with definition-based expansion, and constructs responses by similarity-guided candidate selection. The system may be viewed as a rule extractor operating over a token graph rather than a conventional classifier. I formalize the knowledge base, definition expansion, candidate scoring, repetition control, English-rule heuristics, and response construction mechanisms used by ChatIPC. I further situate the method within the literature on rule extraction, decision tree induction, association rules, interpretable machine learning, and sequence …

Class path: 68-XX — Computer science › 68Txx — Artificial intelligence › 68T05 — Learning and adaptive systems in artificial intelligence

**Work 2.** Neural Variance-aware Dueling Bandits with Deep Representation and Shallow Exploration

> We introduce the first variance-aware algorithms for contextual dueling bandits that leverage shallow exploration strategies with neural networks for nonlinear utility approximation. A key theoretical challenge is the absence of a closed-form estimator, which led prior work to require an extremely large network width $m$ (i.e., $m = \widetilde{\Omega}(T^{14})$). We address this constraint with a novel analytical approach that combines iterative self-improvement with spectral analysis. Our analysis significantly reduces the network width requirement to $m = \widetilde{\Omega}(T^{6})$, and shows that our algorithms achieve a sublinear regret of $\widetilde{\mathcal{O}}(d\sqrt{\sum_{t=1}^{T} \sigma_t^2} + \sqrt{dT})$ under both UCB and TS frameworks. Empirical results show that the proposed algorithms are not only computationally efficient and exhibit sublinear regret in practical …

Class path: 68-XX — Computer science › 68Txx — Artificial intelligence › 68T05 — Learning and adaptive systems in artificial intelligence

## 98. uid `29ccdc44b3`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** DriftWorld: Fast World Modeling through Drifting

> Predictive world models enable robots to simulate the visual outcomes of their actions, but state-of-the-art diffusion-based models remain costly because generating each rollout requires multi-step iterative denoising. We introduce DriftWorld, an action-conditioned world model based on drifting generative models. DriftWorld learns a conditional drift during training, enabling it to generate future observations for a given action sequence in a single forward pass during inference. Across Bridge-V2, RT-1, Language Table, Push-T, and Robomimic, DriftWorld runs at over 40 fps and is 12+ times faster than diffusion-based baselines, while matching or improving their visual generation quality. This makes DriftWorld an efficient world model for robot simulation and further enables downstream applications including inference-time action search and offline policy evaluation.

Class path: B — Philosophy. Psychology. Religion › BS — The Bible › BS701-1830 — Old Testament › BS1109.22-1199 — Works about the Old Testament › BS1199.A-1199.Z — Topics (not otherwise provided for), A-Z › BS1199.W74 — World

**Work 2.** Diffusion-Guided Multi-Arm Motion Planning

> Multi-arm motion planning is fundamental for enabling arms to complete complex long-horizon tasks in shared spaces efficiently but current methods struggle with scalability due to exponential state-space growth and reliance on large training datasets for learned models. Inspired by Multi-Agent Path Finding (MAPF), which decomposes planning into single-agent problems coupled with collision resolution, we propose a novel diffusion-guided multi-arm planner (DG-MAP) that enhances scalability of learning-based models while reducing their reliance on massive multi-arm datasets. Recognizing that collisions are primarily pairwise, we train two conditional diffusion models, one to generate feasible single-arm trajectories, and a second, to model the dual-arm dynamics required for effective pairwise collision resolution. By integrating these specialized generative models within a MAPF-inspired …

Class path: M — Music › MT — Instruction and study › MT240.A-240.Z — Other, A-Z › MT240.A7 — Arm

## 99. uid `b8b20f2fac`

Scheme: OpenAlex Topics

**Work 1.** NAE: Normalizing AutoEncoder

> We consider the setting of Normalizing flows with approximate inverses, an established paradigm spanning both full-dimensional ($d=D$) and bottleneck ($d<D$) settings, and group these models under the term flow autoencoders. We present a theoretical investigation into their training dynamics and prove that the proposed loss used by existing approaches is suboptimal; specifically, both encoder and decoder surrogates must be optimized in alignment with reconstruction loss. Guided by these insights, we propose Normalizing Autoencoder (NAE), which employs a novel conditional loss that aligns the surrogate loss gradient with that of reconstruction loss, directly improving upon the current standard. Extensive experiments across molecule generation, tabular data, and image benchmarks demonstrate that NAE achieves state of the art performance. Our work highlights the importance of loss …

Class path: D3 — Physical Sciences › F15 — Chemical Engineering › S1507 — Fluid Flow and Transfer Processes

**Work 2.** Soliton Dynamics for the Cubic-Quintic Nonlinear Schr\"odinger Equation with a Potential

> We study the semiclassical dynamics of solitary waves for the three-dimensional cubic--quintic nonlinear Schr\"odinger equation with an external potential. For initial data given by a modulated free ground state $P_\omega$, we prove that the wave-packet center follows the associated classical Hamiltonian flow. For $\omega\in\mathcal I$, we obtain qualitative persistence near the soliton orbit. Under the additional positive-slope condition $\omega\in\mathcal I_+$, we establish an $O(\eps)$ approximation in the scaled $H^1$ norm on every fixed time interval. We also derive $O(\eps^2)$ concentration estimates for the mass, momentum, and center of mass.

Class path: D3 — Physical Sciences › F15 — Chemical Engineering › S1507 — Fluid Flow and Transfer Processes

## 100. uid `b6aaffe40c`

Scheme: Free Decimal Correspondence

**Work 1.** Neural Quantum States in Mixed Precision

> Scientific computing has long relied on double precision (64-bit floating point) arithmetic to guarantee accuracy in simulations of real-world phenomena. However, the growing availability of hardware accelerators such as Graphics Processing Units (GPUs) has made low-precision formats attractive due to their superior performance, reduced memory footprint, and improved energy efficiency. In this work, we investigate the role of mixed-precision arithmetic in neural-network based Variational Monte Carlo (VMC), a widely used method for solving computationally otherwise intractable quantum many-body systems. We first derive general analytical bounds on the error introduced by reduced precision on Metropolis-Hastings MCMC, and then empirically validate these bounds on the use-case of VMC. We demonstrate that significant portions of the algorithm, in particular, sampling the quantum state, can …

Class path: 500 — Science; Natural history › 510 — Mathematics › 511 — Arithmetic

**Work 2.** The cost of simulating classically tractable quantum circuits and dynamics

> Determining whether a quantum evolution can be efficiently simulated classically is central to understanding the boundary between classical and quantum computation. However, polynomial-time simulability is an asymptotic statement, and does not by itself determine whether the (quantum-inspired) classical simulation is actually practical. Indeed, different polynomial scalings can lead to vastly different computational costs, particularly when expensive preprocessing or quantum data acquisition is required. In this work, we ask whether classically simulable quantum dynamics are in practice more resource-efficient to simulate classically than to execute directly on quantum hardware. We analyze this question using three resource metrics, quantum sample, quantum time, and classical time complexity, for several widely studied classically simulable circuit families. Using representative …

Class path: 300 — Social sciences › 330 — Economics › 338 — Industries; Prices; Microeconomics

## 101. uid `42ccebf576`

Scheme: arXiv category taxonomy

**Work 1.** Drug-Target Interaction Prediction via Hierarchical Sequential Cross-Attention over Chemical and Protein Language Models

> Predicting Drug-Target Interactions~(DTIs) is a central task in computational drug discovery, with direct applications in virtual screening, drug repurposing, and therapeutic candidate prioritization. Although recent deep learning methods have improved DTI prediction, many sequence-based models still process drugs and proteins independently and only combine their representations at a late prediction stage. This limits their ability to explicitly model cross-molecular dependencies between chemical substructures and protein sequence regions. In this paper, we propose a sequence-only DTI prediction architecture that combines two pre-trained language models, ChemBERTa for drug SMILES strings and ESM-2 for protein amino acid sequences, with a hierarchical interaction module. The proposed model first extracts contextual representations using pre-trained encoders, then applies 1D convolutional …

Class path: grp_cs — Computer Science › cs — Computer Science › cs.LG — Machine Learning

**Work 2.** General-purpose LLMs as Constrained Crystal Composition Generators

> The targeted discovery of inorganic materials remains challenging due to the vastness of compositional design spaces and the high cost of exhaustive screening. Task-specific generative artificial intelligence represents a particularly efficient alternative to screening, yet demands tedious collection of training data before providing real benefit. General-purpose large language models (LLMs) have recently shown tremendous potential for the targeted generation of single, optimal materials compositions without the need for task-specific fine-tuning. However, it is unclear whether LLMs generally pose an advantage compared to specialized generative models, in particular in large design spaces. Here, we demonstrate that such models are capable of covering entire regions of the targeted property space effectively and systematically. Using Elpasolite materials as an established benchmark for …

Class path: grp_physics — Physics › cond-mat — Condensed Matter › cond-mat.mtrl-sci — Materials Science

## 102. uid `913ed5b2ac`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** Neutron Star Equation of State via Physics Informed Neural Network

> We present the first application, to the best of our knowledge, of Physics-Informed Neural Networks (PINNs) to the neutron star equation-of-state (EOS) inverse problem. Two interacting networks -- one representing the EOS $P(\varepsilon)$ as a continuous, non-parametric function, the other solving the Tolman-Oppenheimer-Volkoff (TOV) equations -- are trained jointly on NICER X-ray timing posteriors and pulsar mass measurements. The TOV equations enter as a mean-square ODE residual enforced via automatic differentiation at every training step, rooted in the Neural Differential Equation framework. The inferred EOS satisfies nuclear saturation properties, causality, and perturbative QCD bounds simultaneously; $\chi$EFT consistency at $1$--$2\rhoz$ emerges without explicit enforcement, providing a non-trivial self-consistency check. Across $N=15$ independent training runs, we find a neutron …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.01 — Applied mathematics, analysis and probability › 02.01.01.04 — Stochastic and fractional differential equations

**Work 2.** Specific heat and susceptibility of S=1/2 antiferromagnets on square, triangular, and kagome lattices

> We study the temperature dependence of the thermodynamic properties of spin-1/2 antiferromagnets on two-dimensional lattices. Our analysis employs the sine-square deformation (SSD), in which a real-space envelope function is applied to the Hamiltonian so that the local energy scale is smoothly reduced to zero at the system boundaries. The quantum eigenstates of the SSD Hamiltonian exhibit bulk-like behavior near the system center, effectively mimicking the thermodynamic limit even in small finite-size calculations. Using these fictitious bulk states, we compute the energy density, specific heat, and magnetic susceptibility as functions of temperature. We find that both the triangular- and kagome-lattice antiferromagnets show either a shoulder or a pronounced double-peak structure in the low-temperature specific heat, whereas the kagome case particularly shows a strong enhancement of …

Class path: 02 — Mathematics and physical sciences

## 103. uid `6e7ac05c67`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Isoperimetric inequalities and sharp upper bounds for Aharonov-Bohm eigenvalues on surfaces

> We consider the first eigenvalue of the magnetic Laplacian with zero magnetic field on simply connected compact surfaces and we establish isoperimetric inequalities and upper bounds in terms of a bound on the gaussian curvature. As a corollary, we prove that among all simply connected spherical domains of fixed area, the first eigenvalue is maximal for a geodesic disk with the pole of the magnetic potential at its center; also, for the sphere punctured at two points, the first eigenvalue is maximal when the punctures are antipodal.

Class path: 35-XX — Partial differential equations › 35Pxx — Spectral theory and eigenvalue problems for partial differential equations › 35P15 — Estimates of eigenvalues in context of PDEs

**Work 2.** Sharp bounds for higher Steklov-Dirichlet eigenvalues on domains with spherical holes

> We consider mixed Steklov-Dirichlet eigenvalue problem on smooth bounded domains in Riemannian manifolds. Under certain symmetry assumptions on multiconnected domains in $\mathbb{R}^{n}$ with a spherical hole, we obtain isoperimetric inequalities for $k$-th Steklov-Dirichlet eigenvalues for $2 \leq k \leq n+1$. We extend Theorem 3.1 of \cite{gavitone2023isoperimetric} from Euclidean domains to domains in space forms, that is, we obtain sharp lower and upper bounds of the first Steklov-Dirichlet eigenvalue on bounded star-shaped domains in the unit $n$-sphere and in the hyperbolic space.

Class path: 58-XX — Global analysis, analysis on manifolds › 58Jxx — Partial differential equations on manifolds; differential operators › 58J50 — Spectral problems; spectral geometry; scattering theory on manifolds

## 104. uid `d10f2608df`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** Preemptive Scheduling for Age of Job Minimization in Task-Specific Machine Networks

> We consider a time-slotted job-assignment system consisting of a central server, $N$ task-specific networks of machines, and multiple users. Each network specializes in executing a distinct type of task. Users stochastically generate jobs of various types and forward them to the central server, which routes each job to the appropriate network of machines. Due to resource constraints, the server cannot serve all users' jobs simultaneously, which motivates the design of scheduling policies with possible preemption. To evaluate scheduling performance, we introduce a novel timeliness metric, the age of job, inspired by the well-known metric, the age of information. We study the problem of minimizing the long-term weighted average age of job. We first propose a max-weight policy by minimizing the one-step Lyapunov drift and then derive the Whittle index (WI) policy when the job completion …

Class path: r — rituals › ra — attributes of #ra › rab — historical periods › rabX — millennia › rabXX — centuries › rabXXX — decades › rabXXXX — years › rabXXXXX — months › rabXXXXXX — dates › rabXXXXXXX — times

**Work 2.** Shifted Equivalent Sources and FFT acceleration for Periodic Scattering Problems including Wood Anomalies

> This paper introduces a fast algorithm, applicable throughout the electromagnetic spectrum, for the numerical solution of problems of scattering by periodic surfaces in two-dimensional space. The proposed algorithm remains highly accurate and efficient for challenging configurations including randomly rough surfaces, deep corrugations, large periods, near grazing incidences, and, importantly, Wood-anomaly resonant frequencies. The proposed approach is based on use of certain `shifted equivalent sources' which enable FFT acceleration of a Wood-anomaly-capable quasi-periodic Green function introduced recently (Bruno and Delourme, Jour. Computat. Phys., 262--290, 2014). The Green-function strategy additionally incorporates an exponentially convergent shifted version of the classical spectral series for the Green function. While the computing-cost asymptotics depend on the asymptotic …

Class path: r — rituals › ra — attributes of #ra › rab — historical periods › rabX — millennia › rabXX — centuries › rabXXX — decades › rabXXXX — years › rabXXXXX — months › rabXXXXXX — dates › rabXXXXXXX — times › rabXXXXXXXX — minutes › rabXXXXXXXXX — seconds

## 105. uid `0df444e97a`

Scheme: OpenAlex Topics

**Work 1.** AI-Driven Framework for Adaptive Water Network Management with Proof-of-Concept Implementation: Addressing Non-Revenue Water in Jordan

> Jordan faces severe water scarcity with 50\% of water produced is lost to leakage, theft and metering issues also known as non-revenue water (NRW). Traditional reactive approaches have proven insufficient for sustained NRW reduction. This paper proposes an intelligent framework integrating EPANET hydraulic modeling, digital twin technology, SCADA systems, and large language model (LLM)-based AI agents for continuous network monitoring and adaptive decision-making. The system combines real-time data streams with physics-based simulation to detect anomalies, employing retrieval-augmented generation (RAG) for policy interpretation and function calling for network control. A proof-of-concept implementation validates technical feasibility using EPYT with offline LLMs (llama3.1:8b via Ollama) on a 1,164-junction Amman district network. The system demonstrates automated hydraulic simulation, …

Class path: D3 — Physical Sciences › F17 — Computer Science › S1702 — Artificial Intelligence › T10028 — Topic Modeling

**Work 2.** A Barrier-Metric First-Order Method for Linearly Constrained Bilevel Optimization

> We study bilevel optimization with a fixed polyhedral lower feasible set. Such problems are challenging for two reasons: active-set changes can make the upper objective nonsmooth, and existing hypergradient methods typically require lower-Hessian inversions or equivalent linear solves, which are computationally expensive. To address these issues, we adopt a logarithmic barrier smoothing of the lower problem to obtain a differentiable approximation of the constrained bilevel objective, and develop a proxy-gradient algorithm for the resulting barrier-smoothed surrogate. The algorithm uses only gradients of the upper and lower objectives; its only second-order object is the explicit logarithmic barrier Hessian determined by the fixed polyhedral constraints. Barrier smoothing restores differentiability, but Euclidean smoothness constants are not uniformly bounded near the boundary. We …

Class path: D3 — Physical Sciences › F26 — Mathematics › S2608 — Geometry and Topology › T12396 — Advanced Differential Equations and Dynamical Systems

## 106. uid `13d3e2498b`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** Cohomology of Flag Superschemes and Syzygies of Compositional Varieties

> We study the coherent cohomology of partial flag supervarieties and calculate the sheaf cohomology of the structure sheaves of four new infinite families of flag supervarieties. We introduce compositional varieties, a generalization of determinantal varieties obtained by imposing rank conditions on compositions of a pair of maps. We study geometric properties of compositional varieties and, in cases of interest, compute their $\mathrm{Tor}$-groups. For each of the four families of flag supervarieties, we show that the graded sheaf cohomology is isomorphic to the tensor product of the singular cohomology ring of an ordinary partial flag variety and the graded $\mathrm{Tor}$-groups of a compositional variety.

Class path: 14-XX — Algebraic geometry › 14Mxx — Special varieties › 14M30 — Supervarieties

**Work 2.** Stratification of $\mathrm{AGL}_r(\mathbb{C})$-representation varieties of twisted Hopf links

> We provide a stratification of the $\mathrm{AGL}_r(\mathbb{C})$-representation variety of the fundamental group of the complement of a twisted Hopf link in terms of a stratification of the corresponding $\mathrm{GL}_r(\mathbb{C})$-representation variety. For ranks $1$ and $2$, we explicitly describe this stratification and compute the motives of these varieties in terms of the Lefschetz motive $q=[\mathbb{C}]$ in the Grothendieck ring of complex algebraic varieties $K_0(\mathbf{Var}_{\mathbb{C}})$.

Class path: 57-XX — Manifolds and cell complexes › 57Kxx — Low-dimensional topology in specific dimensions › 57K31 — Invariants of 3-manifolds (including skein modules, character varieties)

## 107. uid `5cab9764cb`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** FlippedRAG: Black-Box Opinion Manipulation Adversarial Attacks to Retrieval-Augmented Generation Models

> Retrieval-Augmented Generation (RAG) enriches LLMs by dynamically retrieving external knowledge, reducing hallucinations and satisfying real-time information needs. While existing research mainly targets RAG's performance and efficiency, emerging studies highlight critical security concerns. Yet, current adversarial approaches remain limited, mostly addressing white-box scenarios or heuristic black-box attacks without fully investigating vulnerabilities in the retrieval phase. Additionally, prior works mainly focus on factoid Q&A tasks, their attacks lack complexity and can be easily corrected by advanced LLMs. In this paper, we investigate a more realistic and critical threat scenario: adversarial attacks intended for opinion manipulation against black-box RAG models, particularly on controversial topics. Specifically, we propose FlippedRAG, a transfer-based adversarial attack against …

Class path: 01 — Computing, AI and engineering › 01.01 — Language models, security and computing systems › 01.01.02 — Security, privacy and software › 01.01.02.04 — Adversarial attacks and defences

**Work 2.** DeepContext: Stateful Real-Time Detection of Multi-Turn Adversarial Intent Drift in LLMs

> While Large Language Model (LLM) capabilities have scaled, safety guardrails remain largely stateless, treating multi-turn dialogues as a series of disconnected events. This lack of temporal awareness facilitates a "Safety Gap" where adversarial tactics, like Crescendo and ActorAttack, slowly bleed malicious intent across turn boundaries to bypass stateless filters. We introduce DeepContext, a stateful monitoring framework designed to map the temporal trajectory of user intent. DeepContext discards the isolated evaluation model in favor of a Recurrent Neural Network (RNN) architecture that ingests a sequence of fine-tuned turn-level embeddings. By propagating a hidden state across the conversation, DeepContext captures the incremental accumulation of risk that stateless models overlook. Our evaluation demonstrates that DeepContext significantly outperforms existing baselines in …

Class path: 01 — Computing, AI and engineering

## 108. uid `eef033ade6`

Scheme: UNESCO Thesaurus

**Work 1.** A sensitivity analysis of interstellar ice chemistry in astrochemical models

> Astrochemical models are essential to bridge the gap between the timescales of reactions, experiments, and observations. Ice chemistry in these models experiences a large computational complexity as a result of the many parameters required for the modeling of chemistry occurring on these ices, such as binding energies and reaction energy barriers. Many of these parameters are poorly constrained, and accurately determining all would be too costly. We aim to find out which parameters describing ice chemistry have a large effect on the calculated abundances of ices for different prestellar objects. Using Monte Carlo sampled binding energies, diffusion barriers, desorption and diffusion prefactors, and reaction energy barriers, we determined the sensitivity of the abundances of the main ice species calculated with UCLCHEM, an astrochemical modeling code, on each of these parameters. We do …

Class path: 2 — Science › 2.45 — Meteorology › c184 — Hydrometeorology › c11526 — Ice

**Work 2.** Demo: tfdrift - A Severity Taxonomy and Risk Classification Framework for Infrastructure Drift Detection

> Infrastructure as Code (IaC) tools like Terraform have become the standard for declarative cloud resource management, yet configuration drift, where deployed infrastructure diverges from its declared state, remains a persistent operational and security challenge. Current detection approaches treat all changes equivalently, contributing to alert fatigue that causes operators to miss security-critical modifications. We propose a generalized severity taxonomy for infrastructure drift that classifies changes into four risk tiers based on resource type and attribute-level impact. We implement this taxonomy in tfdrift, an open-source classification framework with 60+ configurable rules covering AWS, Azure, and GCP resource patterns (evaluation reported here is AWS-focused). Evaluation across 150+ AWS Terraform workspaces demonstrates that severity filtering reduces alert volume by 73% while …

Class path: 2 — Science › 2.05 — Scientific approach › c106 — Scientific methods › c16291 — Typology

## 109. uid `2266c0931a`

Scheme: OpenAlex Topics

**Work 1.** Integrated Alchemical and Conformational Enhanced Sampling for Solvation Free Energy Calculations

> Accurate solvation free energies from molecular dynamics simulations require efficient sampling of coupled slow variables, including solvent coordinates, solute conformational modes, and the alchemical coordinate $\lambda$. Here, we develop a $\lambda$-dynamics framework that combines mass scaling, on-the-fly probability enhanced sampling (OPES), and driven adiabatic free energy dynamics (d-AFED) to address these sampling challenges within a unified protocol. For rigid organic solutes, Hamiltonian replica exchange with mass scaling is first used to quantify the effect of octanol solvent relaxation. Reducing all octanol atomic masses by a factor of ten accelerates convergence by more than fivefold while preserving equilibrium solvation free energies. These calculations then provide reference benchmarks for $\lambda$-OPES, a dual-bias $\lambda$-dynamics strategy that combines the …

Class path: D3 — Physical Sciences › F21 — Energy

**Work 2.** A Kernel Score Perspective on Forecast Disagreement and the Linear Pool

> This paper generalizes several results on linear pooling from squared error loss to all kernel scores. The latter are a rich family of scoring rules that covers point and distribution forecasts for univariate and multivariate, discrete and continuous settings. Its members include the Continuous Ranked Probability Score for univariate distribution forecasting and the Energy Score for multivariate distribution forecasting. Our results indicate that forecast disagreement (measured as the average pairwise divergence of all component distributions) has important implications for the linear pool's performance. The results are useful for understanding and designing linear pools in general combination settings. In particular, they motivate using the linear pool (as opposed to other combination formulas) and yield a novel condition under which equal combination weights are optimal under a given …

Class path: D3 — Physical Sciences › F21 — Energy

## 110. uid `b9b8e83719`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** Abundances in 78 metal-rich bulge spheroid stars from APOGEE

> The inner Galaxy is the most complex region of the Milky Way, comprising the bulge, inner thin and thick discs, and inner halo; the formation of the bar transferred gas and stars from the disc inward. Accretion of dwarf galaxies also occurred over the Galaxy's lifetime, merging with the original bulge. In this work, we constrain the metal-rich stars of the earliest spheroidal bulge. To study the oldest bulge stars, distributed in a spheroid, we applied kinematical and dynamical criteria in the metal-rich range [Fe/H] > -0.8. This complements our previous analysis of a symmetric sample with [Fe/H] < -0.8. We derived individual abundances through spectral synthesis for C, N, O, Al, P, S, K, Mn, and Ce using stellar parameters from APOGEE DR17, and compared the results with literature data and chemical-evolution models. The alpha elements Mg, Si, and Ca, and iron-peak elements V, Cr, Co, …

Class path: 02 — Mathematics and physical sciences › 02.02 — Physics › 02.02.03 — Astrophysics and particle physics

**Work 2.** Carbon-Enhanced Metal-Poor Star Candidates in the Milky Way from J-PLUS and S-PLUS

> Recent large-scale multi-band photometric surveys now enable elemental-abundance estimates for millions of stars with accuracies approaching those of low- to medium-resolution spectroscopy. Using [Fe/H] and [C/Fe] estimates derived from the Javalambre Photometric Local Universe Survey (J-PLUS) DR3 and the Southern Photometric Local Universe Survey (S-PLUS) DR4, which together cover $\sim$6,200 deg$^2$ of the sky, we identify large numbers of carbon-enhanced metal-poor (CEMP) stars in the Milky Way. After applying data-quality cuts and evolutionary corrections to the carbon-abundance estimates, we construct a combined J/S-PLUS sample of $\sim$6.40 million stars and identify $\sim$104,900 CEMP candidates, roughly twice the number of CEMP candidates identified from Gaia XP spectra by Lucey et al. We photometrically confirm that the absolute carbon abundance $A$(C) separates CEMP stars into …

Class path: 02 — Mathematics and physical sciences › 02.01 — Mathematics › 02.01.02 — Pure mathematics › 02.01.02.03 — Groups, Lie algebras and topology

## 111. uid `f5cb3284b6`

Scheme: Mathematics Subject Classification 2020 (MSC2020)

**Work 1.** New logarithmic power nonlinear Schr\"odinger equations with super-Gaussons

> We introduce a new class of nonlinear Schr\"odinger equations with a logarithmic-power nonlinearity that admits exact localized solutions of super-Gaussian form. The resulting stationary states possess flat-top profiles with sharp edges and are referred to as super-Gaussons, in analogy with the Gaussian Gaussons of the classical logarithmic NLS (log-NLS). The model, which we call the logarithmic-power NLS (logp-NLS), is parameterized by an exponent $p\geq1$ that controls the degree of flatness of the soliton core and the sharpness of its decay. Mathematically, $p$ interpolates between the standard log-NLS ($p=1$) and increasingly flat-top profiles as $p$ increases, while physically it governs the stiffness of an underlying logarithmic-power compressibility law. The proposed equation is constructed so as to admit super-Gaussian stationary states and can be interpreted a posteriori within …

Class path: 35-XX — Partial differential equations › 35Qxx — Partial differential equations of mathematical physics and other areas of application › 35Q55 — NLS equations (nonlinear Schrödinger equations)

**Work 2.** Asymptotics of Titchmarsh--Weyl functions near the real axis andan application to the KdV hierarchy

> We establish high-energy asymptotic expansions of Titchmarsh--Weyl functions for one-dimensional Schr\"odinger and Dirac operators in regions whose boundaries approach the spectrum at a prescribed polynomial rate. For bounded potentials with bounded derivatives, the expansions remain uniform up to these boundaries, with an explicit loss in the remainder determined by the rate of approach. We treat both self-adjoint Dirac operators and non-self-adjoint Dirac operators with skew-adjoint potential matrices, keeping track of the distinct half-planes in which the two scalar Weyl coordinates are naturally defined. As an application of the Schr\"odinger expansion, we verify the high-energy hypothesis in Kotani's construction of KdV flows: for every odd integer $p\geq3$, each real-valued $q\in W^{2p-1,\infty}(\mathbb{R})$ generates a global classical solution of the member of the KdV hierarchy …

Class path: 34-XX — Ordinary differential equations › 34Bxx — Boundary value problems for ordinary differential equations › 34B20 — Weyl theory and its generalizations for ordinary differential equations

## 112. uid `c3f67b94d5`

Scheme: Library of Congress Classification: complete schedules

**Work 1.** Rationality and symmetry of stable pairs generating series of Fano 3-folds

> The generating series of descendent invariants of stable pairs on 3-folds is conjectured to be rational and to satisfy a $q\leftrightarrow q^{-1}$ symmetry. We prove this conjecture for Fano 3-folds. We utilize the same path of stability conditions that Toda used in his proof of the Calabi--Yau version of the conjecture, relating stable pairs and $L$ invariants, and work of the two authors that allows an extension of Joyce's descendent wall-crossing formula to non-standard hearts of $D^b(X)$. We use Ehrhart theory to deal with the combinatorics coming out of the wall-crossing formula. Furthermore, we specialize the wall-crossing formula to primary insertions and prove a strong rationality result predicted by the Pandharipande--Thomas/Gopakumar--Vafa correspondence.

Class path: N — Fine arts › N8217 — A - Industry › N8217.F615 — Folds

**Work 2.** Covariant linear response theory for a photon gas in curved spacetime

> To overcome the failure of the conventional non-relativistic description of statistics in strong gravitational fields, we develop a covariant theory for a photon gas in curved spacetime, starting from the Boltzmann equation and employing the relaxation time approximation. The transport coefficients are computed up to second order, which are not a massless limit of the massive-particle results due to the fundamentally different phase-space geometry of null particles. At second order, the gravitational field renders the coefficients tensor-valued and induces transverse fluxes, analogous to the Hall effect. Using the first-order results, we derive relativistic generalizations of Fick's and Fourier's laws and obtain the radiation diffusion equation for spherically symmetric accretion disks. This provides a covariant description of radiative transport in strong gravitational fields for …

Class path: Q — Science › QC — Physics › QC770-798 — Nuclear and particle physics. Atomic energy. Radioactivity › QC794.8.A-794.8.Z — Special types of interactions, A-Z › QC794.8.G7 — Gravitational

## 113. uid `aa8493bc71`

Scheme: arXiv category taxonomy

**Work 1.** Evaluating AI Investment Strategies

> We study the problem of auditing a black-box algorithmic decision-maker from observable inputs and outputs alone. Our main result is an exact decomposition: under precisely characterized conditions, the cumulative \emph{regret} of a dynamic policy equals the sum of per-period covariances between the cost vector and the policy's decision. This extends the single-period identity of Aldridge~(2026) to the full multi-period setting of stochastic dynamic programming. We prove the identity holds exactly under i.i.d. costs and mean-unbiased Markov policies, derive closed-form bias corrections for non-stationary and time-varying cases, and establish the discounted-horizon analog. A Bellman recursion for the covariance regret functional connects the result to standard reinforcement learning algorithms; for rolling-window policies, the estimation-error bias is $O(d/w)$. The decomposition has …

Class path: grp_econ — Economics › econ — Economics › econ.EM — Econometrics

**Work 2.** Cost-Ordered Feasibility for Multi-Armed Bandits with Cost Subsidy

> The classic multi-armed bandit (MAB) problem tackles the challenge of accruing maximum reward while making decisions under uncertainty. However, in applications, often the goal is to minimize cost subject to a constraint on the minimum permissible reward, an objective captured by multi-armed bandits with cost-subsidy (MAB-CS). Of interest to this paper is the setting where the quality (reward) constraint is specified relative to the unknown best reward and the cost of each arm is known. We characterize the expected sub-optimal samples required by any policy by proving instance-dependent lower bounds that offer new insight into the problem and are a strict generalization of prior bounds. Then, we propose an algorithm called Cost-Ordered Feasibility (COF) that leverages our insight and intelligently combine samples from all arms to gauge the feasibility of a cheap arm. Thereafter, we …

Class path: grp_cs — Computer Science › cs — Computer Science › cs.LG — Machine Learning

## 114. uid `5e6e15280e`

Scheme: arXiv category taxonomy

**Work 1.** GIM-ENDO: A Multimodal Endoscopic Image and Video Dataset for Gastric Intestinal Metaplasia Morphology and Pathology

> Gastric intestinal metaplasia (GIM) is a precursor lesion to gastric dysplasia and adenocarcinoma whose early detection is crucial for intervening in the carcinogenesis cascade. Artificial intelligence (AI) holds considerable promise for real-time endoscopic detection and characterization of GIM. However, development of reliable AI models has been constrained by the absence of publicly available, histopathologically validated datasets that combine detailed endoscopic annotations, histological subtype (complete and incomplete), standardized grading systems, and normal mucosal patterns. GIM-ENDO was designed to fill this gap. The dataset comprises demographic data, endoscopic findings, histopathological results, and H. pylori status acquired using the Olympus EVIS X1 system with white-light endoscopy (WLE) and image-enhanced endoscopy (IEE), including narrow-band imaging (NBI) and …

Class path: grp_cs — Computer Science › cs — Computer Science › cs.CV — Computer Vision and Pattern Recognition

**Work 2.** GroupNL: Low-Resource and Robust CNN Design over Cloud and Device

> Deploying Convolutional Neural Network (CNN) models on ubiquitous Internet of Things (IoT) devices in a cloud-assisted manner to provide users with a variety of high-quality services has become mainstream. Most existing studies speed up model cloud training/on-device inference by reducing the number of convolution (Conv) parameters and floating-point operations (FLOPs). However, they usually employ two or more lightweight operations (e.g., depthwise Conv, $1\times1$ cheap Conv) to replace a Conv, which can still affect the model's speedup even with fewer parameters and FLOPs. To this end, we propose the Grouped NonLinear transformation generation method (GroupNL), leveraging data-agnostic, hyperparameters-fixed, and lightweight Nonlinear Transformation Functions (NLFs) to generate diversified feature maps on demand via grouping, thereby reducing resource consumption while improving the …

Class path: grp_cs — Computer Science › cs — Computer Science › cs.CV — Computer Vision and Pattern Recognition

## 115. uid `27ddf135b7`

Scheme: OpenAlex Topics

**Work 1.** LTLGuard: Formalizing LTL Specifications with Compact Language Models and Lightweight Symbolic Reasoning

> Translating informal requirements into formal specifications is challenging due to the ambiguity and variability of natural language (NL). This challenge is particularly pronounced when relying on compact (small and medium) language models, which may lack robust knowledge of temporal logic and thus struggle to produce syntactically valid and consistent formal specifications. In this work, we focus on enabling resource-efficient open-weight models (4B--14B parameters) to generate correct linear temporal logic (LTL) specifications from informal requirements. We present LTLGuard, a modular toolchain that combines constrained generation with formal consistency checking to generate conflict-free LTL specifications from informal input. Our method integrates the generative capabilities of model languages with lightweight automated reasoning tools to iteratively refine candidate specifications, …

Class path: D3 — Physical Sciences › F17 — Computer Science › S1703 — Computational Theory and Mathematics › T10142 — Formal Methods in Verification

**Work 2.** Matrix as Plan: Structured Logical Reasoning with Feedback-Driven Replanning

> As knowledge and semantics on the web grow increasingly complex, enhancing Large Language Models (LLMs)' comprehension and reasoning capabilities has become particularly important. Chain-of-Thought (CoT) prompting has been shown to enhance the reasoning capabilities of LLMs. However, it still falls short on logical reasoning tasks that rely on symbolic expressions and strict deductive rules. Neuro-symbolic methods address this gap by enforcing formal correctness through external solvers. Yet these solvers are highly format-sensitive, and small instabilities in model outputs can lead to frequent processing failures. The LLM-driven approaches avoid parsing brittleness, but they lack structured representations and process-level error-correction mechanisms. To further enhance the logical reasoning capabilities of LLMs, we propose MatrixCoT, a structured CoT framework with a matrix-based …

Class path: D4 — Health Sciences › F27 — Medicine › S2728 — Neurology

## 116. uid `b1fc7cbbd2`

Scheme: Learned general classification (similarity- and navigation-optimized prototype)

**Work 1.** Semantic Frame Interpolation

> Generating intermediate video content of varying lengths based on given first and last frames, along with text prompt information, offers significant research and application potential. However, traditional frame interpolation tasks primarily focus on scenarios with a small number of frames, no text control, and minimal differences between the first and last frames. Recent community developers have utilized large video models represented by Wan to endow frame-to-frame capabilities. However, these models can only generate a fixed number of frames and often fail to produce satisfactory results for certain frame lengths, while this setting lacks a clear official definition and a well-established benchmark. In this paper, we first propose a new practical Semantic Frame Interpolation (SFI) task from the perspective of academic definition, which covers the above two settings and supports …

Class path: 01 — Computing, AI and engineering › 01.02 — Vision, robotics, dynamics and medicine › 01.02.01 — Computer vision and multimodal models › 01.02.01.03 — Video, 3D reconstruction and tracking

**Work 2.** Omni2Sound: Towards Unified Video-Text-to-Audio Generation

> Training a unified model integrating video-to-audio (V2A), text-to-audio (T2A), and joint video-text-to-audio (VT2A) generation offers significant application flexibility, yet faces two unexplored foundational challenges: (1) the scarcity of high-quality audio captions with tight V-A-T alignment, leading to severe semantic conflict between multimodal conditions, and (2) cross-task and intra-task competition, manifesting as an adverse V2A-T2A performance trade-off and modality bias in the VT2A task. First, to address data scarcity, we introduce SoundAtlas, a large-scale dataset (470k pairs) that significantly outperforms existing benchmarks and even human experts in quality. Powered by a novel agentic pipeline, it integrates Vision-to-Language Compression to mitigate visual bias of MLLMs, a Junior-Senior Agent Handoff for a 5$\times$ cost reduction, and rigorous Post-hoc Filtering to …

Class path: 01 — Computing, AI and engineering

## 117. uid `66295cecd7`

Scheme: Free Decimal Correspondence

**Work 1.** Transfinitely iterated wild sets

> In this paper, we study homotopical analogues of the Cantor-Bendixson derivative. For each $n\geq 0$, the "$\pi_n$-wild set" $\mathbf{w}_n(X)$ of a topological space $X$ is the subspace of $X$ consisting of the points at which there exists a shrinking sequence of essential based maps $S^n\to X$. Since the operator $\mathbf{w}_n$ permits iteration, every given space $X$ yields a descending transfinite sequence of nested subspaces $\{\mathbf{w}_n^{\kappa}(X)\}_{\kappa}$ that stabilizes at some smallest ordinal $\mathbf{wrk}_n(X)$ called the "$\pi_n$-wild rank" of $X$. We show that the entire transfinite sequence $\{ho(\mathbf{w}_n^{\kappa}(X))\}_{\kappa}$ of homotopy types is a homotopy invariant of $X$ and that $\mathbf{wrk}_n(X)$ can be an arbitrary countable ordinal when $X$ is an $n$-dimensional Peano continuum. It remains open if there exists a continuum $X$ with uncountable …

Class path: 600 — Industrial arts; Technology; Engineering › 670 — Manufactures › 676 — Paper; Paper products

**Work 2.** Conformal Symmetry of Relativistic and Nonrelativistic Systems and AdS/CFT Correspondence

> The nonlinear realization of conformal so(2,d) symmetry for relativistic systems and the dynamical conformal so(2,1) symmetry of nonrelativistic systems are investigated in the context of AdS/CFT correspondence. We show that the massless particle in d-dimensional Minkowski space can be treated as the system confined to the border of the AdS_{d+1} of infinite radius, while various nonrelativistic systems may be canonically related to a relativistic (massless, massive, or tachyon) particle on the AdS_2 X S^{d-1}. The list of nonrelativistic systems "unified" by such a correspondence comprises the conformal mechanics model, the planar charge-vortex and 3-dimensional charge-monopole systems, the particle in a planar gravitational field of a point massive source, and the conformal model associated with the charged particle propagating near the horizon of the extreme Reissner-Nordstrom black …

Class path: 500 — Science; Natural history › 530 — Physics › 531 — Mechanics

## 118. uid `875c969a09`

Scheme: Free Decimal Correspondence

**Work 1.** MAT-Fly: An Educational Platform for Simulating Unmanned Aerial Vehicles Aimed to Detect and Track Moving Objects

> The main motivation of this work is to propose a simulation approach for a specific task within the Unmanned Aerial Vehicle (UAV) field, i.e., the visual detection and tracking of arbitrary moving objects. In particular, it is described MAT-Fly, a numerical simulation platform for multi-rotor aircraft characterized by the ease of use and control development. The platform is based on Matlab and the MathWorks Virtual Reality (VR) and Computer Vision System (CVS) toolboxes that work together to simulate the behavior of a quad-rotor while tracking a car that moves along a nontrivial path. The VR toolbox has been chosen due to the familiarity that students have with Matlab and because it does not require a notable effort by the user for the learning and development phase thanks to its simple structure. The overall architecture is quite modular so that each block can be easily replaced with …

Class path: 600 — Industrial arts; Technology; Engineering › 610 — Medicine › 612 — Physiology › 612.8 — Nervous system › 612.84 — Vision

**Work 2.** AI-Increased Talent Retention Strategies: Fostering Long-Term Employee Engagement and Development in Talent Management

> The integration of AI in Talent Management is a change in the way that organizations are designing their strategies for Talent Retention (TR), engagement, and future strategy. New and innovative tools such as predictive models, sentiment analysis, and personalized career planning have come up, and they offer better ways of addressing retention issues, workforce engagement, and, in general, sustainability. Through the application of predictive analytics, organizations can determine employees' likelihood of leaving the organization, who is likely to leave, and when to act, thus minimizing the costs and time associated with the recruitment process and improving performance. Furthermore, AI solutions help the development of individualized learning plans that help to define employees' professional goals and link them with the organization's strategy to encourage the employees' continuous …

Class path: 600 — Industrial arts; Technology; Engineering › 650 — Business › 658 — Management

## 119. uid `9b3ca0471a`

Scheme: Integrative Levels Classification, edition 2 (ILC2)

**Work 1.** A Steenrod square on Khovanov homology and a cup-i product

> Lipshitz-Sarkar defined a stable homotopy type refining Khovanov homology, producing cohomology operations $\text{Sq}^i$ on the Khovanov homology $Kh(L)$ of a link $L$. Later, Mor\'an proposed a sequence of cup-i products on the $\mathbb{F}_2$-coefficient cochain complex of any augmented semi-simplicial object in the Burnside category. Applied to the Khovanov functor, he obtained another sequence of operations $\mathfrak{sq}^n$ on $Kh(L)$, where $\mathfrak{sq}^0$, $\mathfrak{sq}^1$ agree with the usual Steenrod squares. We prove that Lipshitz-Sarkar's $\text{Sq}^2$, the first Steenrod operation that cannot be computed from merely homological data, agrees with Mor\'an's $\mathfrak{sq}^2$.

Class path: a — forms › ag — operations › agp — product

**Work 2.** Distributed sequential federated learning

> The analysis of data stored in multiple sites has become more popular, raising new concerns about the security of data storage and communication. Federated learning, which does not require centralizing data, is a common approach to preventing heavy data transportation, securing valued data, and protecting personal information protection. Therefore, determining how to aggregate the information obtained from the analysis of data in separate local sites has become an important statistical issue. The commonly used averaging methods may not be suitable due to data nonhomogeneity and incomparable results among individual sites, and applying them may result in the loss of information obtained from the individual analyses. Using a sequential method in federated learning with distributed computing can facilitate the integration and accelerate the analysis process. We develop a data-driven method …

Class path: o — instincts › os — behavioural sequences › osu — communication › osua — attributes of #osua › osuas — information

## 120. uid `114090790f`

Scheme: UNESCO Thesaurus

**Work 1.** Complex-Vector Power and Cross-Phase Unbalance in Three-Phase Systems

> Unbalanced three-phase systems still lack a compact phasor-domain representation of power that makes the joint cross-phase voltage-current structure explicit while remaining consistent with established apparent-power definitions. This paper addresses that point through a complex-vector power (CVP) formulation for sinusoidal steady-state operation. The proposed CVP is an element of a direct-sum space whose constituents are the conventional complex-power scalar and a cross-phase unbalance (CPU) vector obtained from the cross product of the voltage and current phasor vectors. The scalar constituent retains the usual active and reactive powers, whereas the CPU vector characterizes their antisymmetric cross-phase relation. Apparent power is thereby separated into intraphase and cross-phase contributions, and its norm is preserved under the power-invariant Fortescue transformation. The …

Class path: 6 — Politics, law and economics › 6.15 — Politics and government › c570 — Political systems › c10266 — State

**Work 2.** Isoholonomic inequalities and speed limits for cyclic quantum systems

> Quantum speed limits set fundamental lower bounds on the time required for a quantum system to evolve between states. Traditional bounds, such as those by Mandelstam-Tamm and Margolus-Levitin, rely on state distinguishability and become trivial for cyclic evolutions where the initial and final states coincide. In this work, we explore an alternative approach based on isoholonomic inequalities, which bound the length of closed trajectories in the state space in terms of their holonomy. Building on a gauge-theoretic framework for mixed-state geometric phases, we extend the concept of isoholonomic inequalities to closed curves of isospectral and isodegenerate density operators. This allows us to derive a new quantum speed limit that remains nontrivial for cyclic evolutions. Our results reveal deep connections between the temporal behavior of cyclic quantum systems and holonomy.

Class path: 6 — Politics, law and economics › 6.15 — Politics and government › c570 — Political systems › c10266 — State
