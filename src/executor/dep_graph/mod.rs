#[cfg(target_os = "dragonos")]
use drstd as std;

use std::sync::Arc;
use std::sync::Mutex;
use std::vec::Vec;

use crate::manager::UnitManager;
use crate::{
    error::runtime_error::{RuntimeError, RuntimeErrorType},
    unit::Unit,
};

pub struct DepGraphNode {
    value: usize,
    edges: Vec<usize>,
    incoming_edges: Vec<usize>,
}

pub struct DepGraph {
    nodes: Vec<DepGraphNode>,
    value: Vec<usize>,
}

// 提供拓扑排序方法，在启动服务时确定先后顺序
impl DepGraph {
    fn new() -> Self {
        return DepGraph {
            nodes: Vec::new(),
            value: Vec::new(),
        };
    }

    pub fn add_node(&mut self, value: usize) -> usize {
        let index = self.nodes.len();
        //如果nodes中已经有了这个value则无需重复添加，直接返回nodes中的value对应的index
        if let Some(idx) = self.value.iter().position(|x| *x == value) {
            return idx;
        }
        //如果value在nodes中不存在，则添加value
        self.nodes.push(DepGraphNode {
            value: value,
            edges: Vec::new(),
            incoming_edges: Vec::new(),
        });
        self.value.push(value);
        return index;
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.nodes[from].edges.push(to);
        self.nodes[to].incoming_edges.push(from);
    }
    pub fn topological_sort(&mut self) -> Result<Vec<usize>, RuntimeError> {
        let mut result = Vec::new();
        let mut visited = Vec::new();
        let mut stack = Vec::new();
        for (i, node) in self.nodes.iter().enumerate() {
            if node.incoming_edges.len() == 0 {
                stack.push(i);
            }
        }
        while stack.len() > 0 {
            let index = stack.pop().unwrap();
            if visited.contains(&index) {
                continue;
            }
            visited.push(index);
            result.push(self.nodes[index].value);
            let len = self.nodes[index].edges.len();
            for i in 0..len {
                let edge = self.nodes[index].edges[i];
                self.nodes[edge].incoming_edges.retain(|&x| x != index);
                if self.nodes[edge].incoming_edges.len() == 0 {
                    stack.push(edge);
                }
            }
        }
        if result.len() != self.nodes.len() {
            return Err(RuntimeError::new(RuntimeErrorType::CircularDependency));
        }
        result.reverse();
        return Ok(result);
    }

    fn add_edges(&mut self, unit: usize, after: &[usize]) {
        //因为service的依赖关系规模不会很大，故先使用递归实现
        //TODO:改递归
        for target in after {
            let s = self.add_node(unit);
            let t = self.add_node(*target);
            self.add_edge(s, t);

            let arc_unit = UnitManager::get_unit_with_id(target).unwrap();
            let unit = arc_unit.lock().unwrap();
            let after = unit.unit_base().unit_part().after();

            self.add_edges(*target, after);
        }
    }

    pub fn construct_graph(unit: &Arc<Mutex<dyn Unit>>) -> DepGraph {
        let mut graph: DepGraph = DepGraph::new();

        let unit = unit.lock().unwrap();
        let uid = unit.unit_id();
        graph.add_node(uid);
        let after = (&unit).unit_base().unit_part().after();
        //递归添加边来构建图
        graph.add_edges(uid, after);
        return graph;
    }
}
