use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_binary, to_binary, Binary, Deps, DepsMut, StdResult};
use cw_storage_plus::Item;
use petgraph::{algo::astar, Graph};

#[cw_serde]
pub enum Pair {
    Fin { address: String },
}

pub const PATHS: Item<Binary> = Item::new("paths_v1");

pub fn add_path(deps: DepsMut, denoms: [String; 2], pair: Pair) -> StdResult<()> {
    let mut graph: Graph<String, Pair> = from_binary(&PATHS.load(deps.storage)?)?;
    let node_a = graph
        .node_indices()
        .find(|node| graph[*node] == denoms[0])
        .unwrap_or_else(|| graph.add_node(denoms[0].clone()));
    let node_b = graph
        .node_indices()
        .find(|node| graph[*node] == denoms[1])
        .unwrap_or_else(|| graph.add_node(denoms[1].clone()));
    graph.add_edge(node_a, node_b, pair);
    PATHS.save(deps.storage, &to_binary(&graph)?)?;
    Ok(())
}

pub fn get_path(deps: Deps, denoms: [String; 2]) -> StdResult<Vec<Pair>> {
    let graph: Graph<String, Pair> = from_binary(&PATHS.load(deps.storage)?)?;
    let node_a = graph.node_indices().find(|node| graph[*node] == denoms[0]);
    let node_b = graph.node_indices().find(|node| graph[*node] == denoms[1]);
    Ok(if let (Some(node_a), Some(node_b)) = (node_a, node_b) {
        let path = astar(&graph, node_a, |n| n == node_b, |_| 0, |_| 0);
        if path.is_none() {
            return Ok(vec![]);
        }
        let path = path.unwrap().1;
        path.windows(2)
            .map(|nodes| graph.find_edge(nodes[0], nodes[1]).unwrap())
            .map(|edge| graph[edge].clone())
            .collect()
    } else {
        vec![]
    })
}

#[cfg(test)]
mod path_tests {
    use cosmwasm_std::testing::mock_dependencies;

    use super::*;

    #[test]
    fn add_path_adds_nodes_and_edge() {
        let mut deps = mock_dependencies();
        let graph = Graph::<String, Pair>::new();
        PATHS
            .save(deps.as_mut().storage, &to_binary(&graph).unwrap())
            .unwrap();
        add_path(
            deps.as_mut(),
            ["denom_a".to_string(), "denom_b".to_string()],
            Pair::Fin {
                address: "address".to_string(),
            },
        )
        .unwrap();
        let graph: Graph<String, Pair> = from_binary(&PATHS.load(&deps.storage).unwrap()).unwrap();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn get_path_returns_empty_if_no_path() {
        let mut deps = mock_dependencies();
        let graph = Graph::<String, Pair>::new();
        PATHS
            .save(deps.as_mut().storage, &to_binary(&graph).unwrap())
            .unwrap();
        let path = get_path(
            deps.as_ref(),
            ["denom_a".to_string(), "denom_b".to_string()],
        )
        .unwrap();
        assert_eq!(path, vec![]);
    }

    #[test]
    fn get_path_returns_path_if_exists() {
        let mut deps = mock_dependencies();
        let graph = Graph::<String, Pair>::new();
        PATHS
            .save(deps.as_mut().storage, &to_binary(&graph).unwrap())
            .unwrap();
        add_path(
            deps.as_mut(),
            ["denom_a".to_string(), "denom_b".to_string()],
            Pair::Fin {
                address: "address".to_string(),
            },
        )
        .unwrap();
        let path = get_path(
            deps.as_ref(),
            ["denom_a".to_string(), "denom_b".to_string()],
        )
        .unwrap();
        assert_eq!(
            path,
            vec![Pair::Fin {
                address: "address".to_string()
            }]
        );
    }

    #[test]
    fn get_path_returns_empty_if_path_does_not_exist() {
        let mut deps = mock_dependencies();
        let graph = Graph::<String, Pair>::new();
        PATHS
            .save(deps.as_mut().storage, &to_binary(&graph).unwrap())
            .unwrap();
        add_path(
            deps.as_mut(),
            ["denom_a".to_string(), "denom_b".to_string()],
            Pair::Fin {
                address: "address_1".to_string(),
            },
        )
        .unwrap();
        add_path(
            deps.as_mut(),
            ["denom_c".to_string(), "denom_d".to_string()],
            Pair::Fin {
                address: "address_2".to_string(),
            },
        )
        .unwrap();
        let path = get_path(
            deps.as_ref(),
            ["denom_a".to_string(), "denom_c".to_string()],
        )
        .unwrap();
        assert_eq!(path, vec![]);
    }

    #[test]
    fn get_path_returns_path_if_path_exists() {
        let mut deps = mock_dependencies();
        let graph = Graph::<String, Pair>::new();
        PATHS
            .save(deps.as_mut().storage, &to_binary(&graph).unwrap())
            .unwrap();
        add_path(
            deps.as_mut(),
            ["denom_a".to_string(), "denom_b".to_string()],
            Pair::Fin {
                address: "address_1".to_string(),
            },
        )
        .unwrap();
        add_path(
            deps.as_mut(),
            ["denom_b".to_string(), "denom_c".to_string()],
            Pair::Fin {
                address: "address_2".to_string(),
            },
        )
        .unwrap();
        let path = get_path(
            deps.as_ref(),
            ["denom_a".to_string(), "denom_c".to_string()],
        )
        .unwrap();
        assert_eq!(
            path,
            vec![
                Pair::Fin {
                    address: "address_1".to_string()
                },
                Pair::Fin {
                    address: "address_2".to_string()
                }
            ]
        );
    }
}
