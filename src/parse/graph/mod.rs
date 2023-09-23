use crate::{
    error::parse_error::{ParseError, ParseErrorType},
    unit::UnitType,
};
use core::slice::SlicePattern;
#[cfg(target_os = "dragonos")]
use drstd as std;
use std::string::{String, ToString};
use std::vec::Vec;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use super::{parse_util::UnitParseUtil, UnitParser};

pub struct GraphNode {
    value: String,
    edges: Vec<usize>,
    incoming_edges: Vec<usize>,
}

pub struct Graph {
    total_edge: u32,
    max_edge: u32,
    nodes: Vec<GraphNode>,
    value: Vec<String>,
}

impl Graph {
    fn new() -> Self {
        return Graph {
            max_edge: 0,
            total_edge: 0,
            nodes: Vec::new(),
            value: Vec::new(),
        };
    }

    pub fn add_node(&mut self, value: &String) -> usize {
        let index = self.nodes.len();
        //如果nodes中已经有了这个value则无需重复添加，直接返回nodes中的value对应的index
        if let Some(idx) = self.value.iter().position(|x| *x == *value) {
            return idx;
        }
        //如果value在nodes中不存在，则添加value
        self.nodes.push(GraphNode {
            value: value.to_string(),
            edges: Vec::new(),
            incoming_edges: Vec::new(),
        });
        self.value.push(value.to_string());
        return index;
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.total_edge += 1;
        self.nodes[from].edges.push(to);
        self.nodes[to].incoming_edges.push(from);
    }
    pub fn topological_sort(&mut self) -> Result<Vec<String>, ParseError> {
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
            result.push(self.nodes[index].value.clone());
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
            return Err(ParseError::new(
                ParseErrorType::ECircularDependency,
                "".to_string(),
                0,
            ));
        }
        result.reverse();
        return Ok(result);
    }

    fn add_edges(&mut self, path: &String, after: Vec<String>) {
        //因为service的依赖关系规模不会很大，故先使用递归实现
        //TODO:改递归
        for target in after {
            let s = self.add_node(&path);
            let t = self.add_node(&target);
            self.add_edge(s, t);
            if self.total_edge > self.max_edge {
                return;
            }
            self.add_edges(&target, Self::parse_after(&target));
        }
    }

    pub fn construct_graph(unit: String) -> Result<Graph, ParseError> {
        //计算整个依赖图中的节点数
        let mut node_num = 1;
        let mut dep = Vec::new();
        Self::get_node_num(&unit, &mut dep, &mut node_num);

        let mut graph: Graph = Graph::new();
        graph.max_edge = node_num * (node_num - 1);

        graph.add_node(&unit);
        let after = Self::parse_after(&unit);
        //递归添加边来构建图
        graph.add_edges(&unit, after);

        if graph.max_edge < graph.total_edge {
            return Err(ParseError::new(
                ParseErrorType::ECircularDependency,
                unit,
                0,
            ));
        }

        return Ok(graph);
    }

    pub fn parse_after(path: &String) -> Vec<String> {
        let mut ret = Vec::new();

        let reader = UnitParser::get_reader(path, UnitType::Unknown).unwrap();

        let mut lines_with_after = Vec::new();

        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                if line.starts_with("After") {
                    lines_with_after.push(line);
                }
            }
        }

        for line in lines_with_after {
            let after = &line.split('=').collect::<Vec<&str>>()[1];
            let units = after.split_whitespace().collect::<Vec<&str>>();

            for unit in units {
                ret.push(String::from(unit));
            }
        }

        ret
    }

    /// ## 获取到unit文件依赖图节点数
    ///
    /// ### param file_path unit文件路径
    ///
    /// ### dependencies 缓存after依赖的容器
    ///
    /// ### total_after_count 返回节点数
    ///
    /// ### return
    fn get_node_num(
        file_path: &str,
        dependencies: &mut Vec<String>,
        total_after_count: &mut u32,
    ) -> Result<(), ParseError> {
        let reader = UnitParser::get_reader(file_path, UnitType::Unknown)?;

        let mut current_after_count = 0;

        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                if line.starts_with("After=") {
                    let dependencies_str = &line[6..];
                    let dependency_list: Vec<&str> = dependencies_str.split_whitespace().collect();

                    for dependency in dependency_list {
                        if dependencies.contains(&dependency.to_string()) {
                            // 循环依赖检查
                            continue;
                        }

                        dependencies.push(dependency.to_string());

                        // 递归解析依赖链
                        Self::get_node_num(dependency, dependencies, total_after_count)?;
                    }

                    current_after_count += 1;
                }
            }
        }

        *total_after_count += current_after_count;

        Ok(())
    }
}
