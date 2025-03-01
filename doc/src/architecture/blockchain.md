# Dynamic Proof of Stake

## blockchain

blockchain $\mathbb{C}$ is a series of epochs, it's a tree of chains, $C_1$, $C_2$, $\dots$, $C_n$, the chain of the max length in $\mathbb{C}$ is the driving chain C

## epoch
is multiple of blocks, some of those block might be empty due to the nature of the leader selection with VRF.


## Genesis block
the first block in the epoch updates the stake for stakeholders, which influences weighted random leader selection algorithm, for epoch j, the pair ($S_j,\eta_j$) is the genesis block's data for n stakeholders of the blockchain:
$$S_j=((U_1,v_1^{vrf},v_1^{kes},v_1^{dsig},s_1),\dots,(U_n,v_n^{vrf},v_n^{kes},v_n^{dsig},s_n)$$
$$\eta_j \leftarrow \{0,1\}^\lambda$$
$$\small\text{\emph{note that new stakeholders need to wait for the next epoch to be added to the genesis block}}$$

## Block
block $\textbf{B}$ is the building block of the blockchain

block $B_{i}=(st,d,sl,B_{\pi},\rho, \sigma_s)$ created for slot i by stakeholder, and slot i leader $U_s$:

$$\textbf{\textcolor{red}{st}}: \text{state of the prebvious block, Hash(head($\mathbb{C}$}$$
$$\textbf{\textcolor{red}{d}}: \text{data held by the block}$$
$$\textbf{\textcolor{red}{sl}}: \text{slot id generated by the beacon}$$
$$\textbf{\textcolor{red}{$B_\pi$}}: \text{proof the stakeholder ${U_s}$ is the owner, $B_{\pi}=(U_s,y,\pi)$, y,$\pi$ are the output of the VRF}$$
$$\textbf{\textcolor{red}{$\rho$}}: \text{random seed for vrf, $\rho=(\rho_y,\rho_{\pi})$}$$
$$\textbf{\textcolor{red}{$\sigma_{s}$}}: \text{owner signature on the block}$$



## leader selection
at the onset of each slot each stakeholder needs to verify if it's the weighted random leader for this slot.
$$y < T_{i}$$
$$\small\text{\emph{check if VRF output is less than some threshold}}$$
this statement might hold true for zero or more stakeholders, thus we might end up with multiple leaders for a slot, and other times no leader.
also note that no one would know who is the leader, how many leaders are there for the slot, until you receive signed block with a proof claiming to be a leader.
$$y = VRF(\eta||sid)$$
$$\small\text{\emph{$\eta$ is random nonce generated from the blockchain, $\textbf{sid}$ is block id}}$$
$$\phi_{f} = 1 - (1-f)^{\alpha_i}$$
$$T_{i} = 2^{l_{VRF}}\phi_{f}(\alpha_i^j)$$
note that $\phi_f(1)=f$, \textbf{f}: the active slot coefficient is the probability that a party holding all the stake will be selected to be a leader.
stakeholder is selected as leader for slot j with probability $\phi_f(\alpha_i)$, $\alpha_i$ is $U_i$ stake.

## leaky non-resettable beacon
built on top of globally synchronized clock, that leaks the nonce $\eta$ of the next epoch a head of time (thus called leaky), non-resettable in the sense that the random nonce is deterministic at slot s, while assuring security against adversary controlling some stakeholders.
for an epoch j, the nonce $\eta_j$ is calculated by hash function H, as:
$$\eta_j = H(\eta_{j-1}||j||v)$$
v is the  concatentation of the value $\rho$ in all blocks from the beginning of epoch $e_{i-1}$ to the slot with timestamp up to $(j-2)R + \frac{16k}{1+\epsilon}$, note that k is a persistence security parameter, R is the epoch length in terms of slots.


# Protocol

# Appendix
This section gives further details about the structures that will be used by the protocol. Since Streamlet consensus protocol will be used at early stages of the Blockchain development, we created hybrid structures, to enable seemless transition from one protocol to the other, without the need of a blockchain forking.

## Blockchain
\begin{tabular}{||c c c||} 
 \hline
 Field & Type & Description \\ [0.5ex] 
 \hline\hline
 blocks & Vec<Block> & Series of blocks consisting the Blockchain. \\ [1ex] 
 \hline
\end{tabular}

## Block
\begin{tabular}{||c c c||} 
 \hline
 Field & Type & Description \\ [0.5ex] 
 \hline\hline
 st & String & Previous block hash. \\ 
 \hline
 sl & u64 & Slot uid, generated by the beacon. \\
 \hline
 txs & Vec<Transaction> & Transactions payload. \\
 \hline
 metadata & Metadata & Additional block information. \\ [1ex] 
 \hline
\end{tabular}

## Metadata
\begin{tabular}{||c c c||} 
 \hline
 Field & Type & Description \\ [0.5ex] 
 \hline\hline
 om & OuroborosMetadata & Block information used by Ouroboros consensus. \\ 
 \hline
 sm & StreamletMetadata & Block information used by Streamlet consensus. \\
 \hline
 timestamp & Timestamp & Block creation timestamp. \\ [1ex] 
 \hline
\end{tabular}

## Ouroboros\_Metadata
\begin{tabular}{||c c c||} 
 \hline
 Field & Type & Description \\ [0.5ex] 
 \hline\hline
 proof & VRFOutput & Proof the stakeholder is the block owner. \\ 
 \hline
 r & Seed & Random seed for VRF. \\
 \hline
 s & Signature & Block owner signature. \\ [1ex] 
 \hline
\end{tabular}


## Streamlet\_Metadata
\begin{tabular}{||c c c||} 
 \hline
 Field & Type & Description \\ [0.5ex] 
 \hline\hline
 votes & Vec<Vote> & Epoch votes for the block. \\ 
 \hline
 notarized & bool & Block notarization flag. \\
 \hline
 finalized & bool & Block finalization flag. \\ [1ex] 
 \hline
\end{tabular}
