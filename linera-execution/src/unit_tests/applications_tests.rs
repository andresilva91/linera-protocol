// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use super::{
    ApplicationRegistry, ApplicationRegistryView, BytecodeId, BytecodeLocation,
    UserApplicationDescription, UserApplicationId,
};
use linera_base::{
    crypto::CryptoHash,
    data_types::{BlockHeight, ChainId, EffectId},
};

fn effect_id(index: usize) -> EffectId {
    EffectId {
        chain_id: ChainId::root(0),
        height: BlockHeight::from(0),
        index,
    }
}

fn bytecode_id(index: usize) -> BytecodeId {
    BytecodeId(effect_id(index))
}

fn app_id(index: usize) -> UserApplicationId {
    UserApplicationId {
        bytecode_id: bytecode_id(0),
        creation: effect_id(index),
    }
}

fn app_description(index: usize, deps: Vec<usize>) -> UserApplicationDescription {
    UserApplicationDescription {
        bytecode_id: bytecode_id(0),
        bytecode_location: location(0),
        creation: effect_id(index),
        initialization_argument: vec![],
        required_application_ids: deps.into_iter().map(app_id).collect(),
    }
}

fn location(operation_index: usize) -> BytecodeLocation {
    BytecodeLocation {
        certificate_hash: CryptoHash::new(&[0u8][..]),
        operation_index,
    }
}

fn registry(graph: impl IntoIterator<Item = (usize, Vec<usize>)>) -> ApplicationRegistry {
    let mut registry = ApplicationRegistry::default();
    for (index, deps) in graph {
        let description = app_description(index, deps);
        registry
            .known_applications
            .insert((&description).into(), description);
    }
    registry
}

#[tokio::test]
async fn test_topological_sort() {
    let mut view = ApplicationRegistryView::new().await;
    view.import(registry([(1, vec![2, 3])])).unwrap();
    assert!(view.find_dependencies(app_id(1)).await.is_err());
    view.import(registry([(3, vec![2]), (2, vec![]), (0, vec![1])]))
        .unwrap();
    let results = view.find_dependencies(app_id(1)).await.unwrap();
    assert_eq!(results, Vec::from_iter([2, 3, 1].into_iter().map(app_id)));
    let results = view.find_dependencies(app_id(0)).await.unwrap();
    assert_eq!(
        results,
        Vec::from_iter([2, 3, 1, 0].into_iter().map(app_id))
    );
}

#[tokio::test]
async fn test_topological_sort_with_loop() {
    let mut view = ApplicationRegistryView::new().await;
    view.import(registry([
        (1, vec![2, 3]),
        (3, vec![1, 2]),
        (2, vec![]),
        (0, vec![1]),
    ]))
    .unwrap();
    let results = view.find_dependencies(app_id(1)).await.unwrap();
    assert_eq!(results, Vec::from_iter([2, 3, 1].into_iter().map(app_id)));
    let results = view.find_dependencies(app_id(0)).await.unwrap();
    assert_eq!(
        results,
        Vec::from_iter([2, 3, 1, 0].into_iter().map(app_id))
    );
}