# Algorithm Overview

Entropy-based strategy with constraint propagation.

-   Compute expected information gain (Shannon entropy) per guess
-   Filter remaining words by feedback-derived constraints
-   Pick highest-entropy guess; repeat until solved

Layers (Clean Architecture):

-   Infrastructure: entropy calculation and ranking
-   Domain: constraint filtering and feedback analysis
-   Application: orchestration and commands
