use std::{fmt::Debug, hash::Hash};

use indexmap::{
    indexmap,
    map::{Entry, IndexMap}
};

/// Starting at `start_node`, return, in arbitrary order, all least-cost success nodes.
///
/// * `neighbours` takes a node `n` and returns an iterator consisting of all `n`'s neighbouring
/// nodes.
/// * `success` takes a node `n` and returns `true` if it is a success node or `false` otherwise.
///
/// This API is roughly modelled after
/// [`astar_bag_collect`](https://docs.rs/pathfinding/0.6.8/pathfinding/fn.astar_bag.html)
/// in the `pathfinding` crate. Unlike `astar_bag_collect`, this `astar_all` does not record the
/// path taken to reach a success node: this allows it to be substantially faster.
pub(crate) fn astar_all<N, FN, FM, FS>(
    start_node: N,
    neighbours: FN,
    merge: FM,
    success: FS
) -> Vec<N>
where
    N: Debug + Clone + Hash + Eq + PartialEq,
    FN: Fn(bool, &N, &mut Vec<(u16, u16, N)>) -> bool,
    FM: Fn(&mut N, N),
    FS: Fn(&N) -> bool
{
    // We tackle the problem in two phases. In the first phase we search for a success node, with
    // the cost monotonically increasing. All neighbouring nodes are stored for future inspection,
    // even if they're of higher cost than the current node. The second phase begins as soon as
    // we've found the first success node. At this point, we still need to search for neighbours,
    // but we can discard any nodes of greater cost than the first success node.
    //
    // The advantage of this two-phase split is that in the second phase we need do substantially
    // less computation and storage of nodes.

    // First phase: search for the first success node.

    let mut scs_nodes = Vec::new();
    // todo is a map from "original node" to "merged node". We never change "original node", but,
    // as we find compatible repairs, continually update merged node. This means that when we pop
    // things off the todo we *must* use "merged node" as our node to work with.
    let mut todo: Vec<IndexMap<N, N>> = vec![indexmap![start_node.clone() => start_node]];
    let mut c: u16 = 0; // What cost are we currently examining?
    let mut next = Vec::new();
    loop {
        if todo[usize::from(c)].is_empty() {
            c = c.checked_add(1).unwrap();
            if usize::from(c) == todo.len() {
                // No success node found and search exhausted.
                return Vec::new();
            }
            continue;
        }

        let n = todo[usize::from(c)].pop().unwrap().1;
        if success(&n) {
            scs_nodes.push(n);
            break;
        }

        if !neighbours(true, &n, &mut next) {
            #[cfg(test)]
            panic!("Timeout occurred.");
            #[cfg(not(test))]
            return Vec::new();
        }
        for (nbr_cost, nbr_hrstc, nbr) in next.drain(..) {
            debug_assert!(nbr_cost.checked_add(nbr_hrstc).unwrap() >= c);
            let off = usize::from(nbr_cost.checked_add(nbr_hrstc).unwrap());
            for _ in todo.len()..off + 1 {
                todo.push(IndexMap::new());
            }
            match todo[off].entry(nbr.clone()) {
                Entry::Vacant(e) => {
                    e.insert(nbr);
                }
                Entry::Occupied(mut e) => {
                    merge(&mut e.get_mut(), nbr);
                }
            }
        }
    }

    // Second phase: find remaining success nodes.
    //
    // Note: There's no point in searching the neighbours of success nodes: the only way they can
    // lead to further success is if they only contain extra (zero-cost, by definition) shifts.
    // That never leads to more interesting repairs from our perspective.

    // Free up all memory except for the cost todo that contains the first success node.
    let mut scs_todo = todo
        .drain(usize::from(c)..usize::from(c) + 1)
        .nth(0)
        .unwrap();
    while !scs_todo.is_empty() {
        let n = scs_todo.pop().unwrap().1;
        if success(&n) {
            scs_nodes.push(n);
            continue;
        }
        if !neighbours(false, &n, &mut next) {
            return Vec::new();
        }
        for (nbr_cost, nbr_hrstc, nbr) in next.drain(..) {
            assert!(nbr_cost + nbr_hrstc >= c);
            // We only need to consider neighbouring nodes if they have the same cost as
            // existing success nodes and an empty heuristic.
            if nbr_cost + nbr_hrstc == c {
                match scs_todo.entry(nbr.clone()) {
                    Entry::Vacant(e) => {
                        e.insert(nbr);
                    }
                    Entry::Occupied(mut e) => {
                        merge(&mut e.get_mut(), nbr);
                    }
                }
            }
        }
    }

    scs_nodes
}

/// Starting at `start_node`, return, in arbitrary order, all least-cost success nodes.
///
/// * `neighbours` takes a node `n` and returns an iterator consisting of all `n`'s neighbouring
/// nodes.
/// * `success` takes a node `n` and returns `true` if it is a success node or `false` otherwise.
///
/// The name of this function isn't entirely accurate: this isn't Dijkstra's original algorithm or
/// one of its well-known variants. However, unlike the astar_all function it doesn't expect a
/// heuristic and it also filters out some duplicates.
pub(crate) fn dijkstra<N, FM, FN, FS>(
    start_node: N,
    neighbours: FN,
    merge: FM,
    success: FS
) -> Vec<N>
where
    N: Debug + Clone + Hash + Eq + PartialEq,
    FN: Fn(bool, &N, &mut Vec<(u16, N)>) -> bool,
    FM: Fn(&mut N, N),
    FS: Fn(&N) -> bool
{
    let mut scs_nodes = Vec::new();
    let mut todo: Vec<IndexMap<N, N>> = vec![indexmap![start_node.clone() => start_node]];
    let mut c: u16 = 0;
    let mut next = Vec::new();
    loop {
        if todo[usize::from(c)].is_empty() {
            c = c.checked_add(1).unwrap();
            if usize::from(c) == todo.len() {
                return Vec::new();
            }
            continue;
        }

        let n = todo[usize::from(c)].pop().unwrap().1;
        if success(&n) {
            scs_nodes.push(n);
            break;
        }

        if !neighbours(true, &n, &mut next) {
            return Vec::new();
        }
        for (nbr_cost, nbr) in next.drain(..) {
            let off = usize::from(nbr_cost);
            for _ in todo.len()..off + 1 {
                todo.push(IndexMap::new());
            }
            match todo[off].entry(nbr.clone()) {
                Entry::Vacant(e) => {
                    e.insert(nbr);
                }
                Entry::Occupied(mut e) => {
                    merge(&mut e.get_mut(), nbr);
                }
            }
        }
    }

    let mut scs_todo = todo
        .drain(usize::from(c)..usize::from(c) + 1)
        .nth(0)
        .unwrap();
    while !scs_todo.is_empty() {
        let n = scs_todo.pop().unwrap().1;
        if success(&n) {
            scs_nodes.push(n);
            continue;
        }
        if !neighbours(false, &n, &mut next) {
            return Vec::new();
        }
        for (nbr_cost, nbr) in next.drain(..) {
            if nbr_cost == c {
                match scs_todo.entry(nbr.clone()) {
                    Entry::Vacant(e) => {
                        e.insert(nbr);
                    }
                    Entry::Occupied(mut e) => {
                        merge(&mut e.get_mut(), nbr);
                    }
                }
            }
        }
    }

    scs_nodes
}
