
## Comparing No Sandwich Swap to Traditional DEXs

| Feature                | No Sandwich Swap      | Traditional DEXs      |
|------------------------|-----------------------|-----------------------|
| **Anti-Front-Running** | Yes                   | No                    |
| **Trade Privacy**      | Enhanced              | Limited               |
| **Order Execution**    | Ordered               | Sequential            |
| **Market Manipulation**| Mitigated             | Vulnerable            |

```mermaid
graph TD;
    subgraph Comparing
        subgraph Traditional AMM
            t_amm_d["Attackers can manipulate the ordered transactions to get MEV profile."]
            
            subgraph Attacker Sandwitch Trader
                Attacker_Buy[Attacker Buy]
                Trader_Buy[Trader Buy]
                Attacker_Sell[Attacker Sell]
            end
        end

        subgraph Anti MEV Swap
            a_mev_s_d["Transactions are orderred and mixed, prevent Attackers to manipulate market price"]
            
            subgraph Ordering Transactions
                Buy1[Buy]
                Sell1[Sell]
                Buy2[Buy]
                Sell2[Sell]
            end
        end
    end

    classDef sell stroke:#f00,color:#f00,stroke-width:3px;
    classDef buy stroke:#0f0,color:#0f0,stroke-width:3px;
    classDef rawText fill: none, stroke: none, font: semi-bold;

    %% Apply classes to specific nodes
    class Attacker_Buy buy;
    class Attacker_Sell sell;

    class Buy1 buy;
    class Buy2 buy;
    class Sell1 sell;
    class Sell2 sell;

    class a_mev_s_d rawText;
    class t_amm_d rawText;
```
