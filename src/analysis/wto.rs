use log::debug;
use rustc_data_structures::graph::WithSuccessors;
use rustc_middle::mir::BasicBlock;
use rustc_middle::mir::{self, Body};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

/// Represents a component of CFG, it is either a single vertex, or a loop circle
#[derive(Clone)]
pub enum WtoComponent {
    Vertex(WtoVertex),
    Circle(WtoCircle),
}

impl Debug for WtoComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WtoComponent::Vertex(v) => write!(f, "{:?}", v),
            WtoComponent::Circle(c) => write!(f, "{:?}", c),
        }
    }
}

/// Represents a single vertex in CFG
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct WtoVertex {
    node: BasicBlock,
}

impl WtoVertex {
    pub fn new(bb: BasicBlock) -> Self {
        Self { node: bb }
    }

    /// Check whether it is the entry node of the CFG
    pub fn is_entry(&self) -> bool {
        self.node == mir::START_BLOCK
    }

    pub fn node(&self) -> BasicBlock {
        self.node
    }
}

impl Debug for WtoVertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.node)
    }
}

/// Represents a loop circle in CFG, a circle may contain vertexes or circles
#[derive(Clone)]
pub struct WtoCircle {
    /// The loop head node
    head: BasicBlock,
    /// The loop body, it has either other vertexes or other circles
    component: Vec<WtoComponent>,
    /// The number of times the circle has been visited
    /// This is used to decide whether to use widening
    num_iter: RefCell<u32>,
}

impl Debug for WtoCircle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = String::from("(");
        r.push_str(format!("{:?} ", self.head()).as_str());
        for comp in &self.component {
            r.push_str(format!("{:?} ", comp).as_str());
        }
        r.pop(); // Remove the last white space
        r.push(')');
        if *self.num_iter.borrow() > 1 {
            r.push_str(format!("^{{{}}}", self.num_iter.borrow()).as_str());
        }
        write!(f, "{}", r)
    }
}

impl IntoIterator for WtoCircle {
    type Item = WtoComponent;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.component.into_iter()
    }
}

impl<'a> IntoIterator for &'a WtoCircle {
    type Item = &'a WtoComponent;
    type IntoIter = std::slice::Iter<'a, WtoComponent>;
    fn into_iter(self) -> Self::IntoIter {
        self.component.iter()
    }
}

impl<'a> IntoIterator for &'a mut WtoCircle {
    type Item = &'a mut WtoComponent;
    type IntoIter = std::slice::IterMut<'a, WtoComponent>;
    fn into_iter(self) -> Self::IntoIter {
        self.component.iter_mut()
    }
}

impl WtoCircle {
    pub fn new(head: BasicBlock, component: Vec<WtoComponent>) -> Self {
        Self {
            head,
            component,
            num_iter: RefCell::new(0),
        }
    }

    pub fn head(&self) -> WtoVertex {
        WtoVertex::new(self.head)
    }

    pub fn inc_iter_num(&self) {
        *(self.num_iter.borrow_mut()) += 1;
    }

    pub fn get_iter_num(&self) -> u32 {
        *self.num_iter.borrow()
    }
}

/// The weak topological order of a CFG
#[derive(Clone)]
pub struct Wto<'tcx> {
    stack: Vec<BasicBlock>,
    dfn: HashMap<BasicBlock, u32>,
    wto_components: Vec<WtoComponent>,
    num: u32,
    cfg: &'tcx Body<'tcx>,
    loop_heads: Vec<BasicBlock>,
    nesting_map: HashMap<WtoVertex, WtoNesting>,
}

impl<'tcx> Debug for Wto<'tcx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = String::new();
        for comp in &self.wto_components {
            r.push_str(format!("{:?} ", comp).as_str());
        }
        r.pop();
        write!(f, "{}", r)
    }
}

/// This is defined so that we can use Body's method directly from Wto
impl<'tcx> Deref for Wto<'tcx> {
    type Target = Body<'tcx>;

    fn deref(&self) -> &Body<'tcx> {
        self.cfg
    }
}

impl<'tcx> Wto<'tcx> {
    pub fn new(cfg: &'tcx Body<'tcx>) -> Self {
        let mut dfn = HashMap::new();
        for (bb, _) in cfg.basic_blocks().iter_enumerated() {
            dfn.insert(bb, 0);
        }
        let mut wto = Self {
            stack: Vec::new(),
            dfn,
            wto_components: Vec::new(),
            num: 0,
            cfg,
            loop_heads: Vec::new(),
            nesting_map: HashMap::new(),
        };

        // Calculate wto
        let mut partition = Vec::new();
        wto.visit(mir::START_BLOCK, &mut partition);
        partition.reverse();
        wto.wto_components = partition;

        // Calculate wto nesting
        let mut wto_nesting_iterator = WtoNestingIterator::new();
        wto_nesting_iterator.visit(&wto);

        debug!("WTO Nesting: {:?}", wto_nesting_iterator);
        wto.nesting_map = wto_nesting_iterator.get_wto_nesting_map();

        wto
    }

    pub fn get_mir(&self) -> &'tcx Body<'tcx> {
        self.cfg
    }

    pub fn components(&self) -> Vec<WtoComponent> {
        self.wto_components.clone()
    }

    fn component(&mut self, vertex: BasicBlock) -> WtoCircle {
        let mut partition = Vec::new();
        for succ in self.cfg.successors(vertex) {
            if self.dfn[&succ] == 0 {
                self.visit(succ, &mut partition);
            }
        }
        // partition.push(Box::new(WtoVertex::new(vertex)));
        partition.reverse();
        WtoCircle::new(vertex, partition)
    }

    fn visit(&mut self, vertex: BasicBlock, partition: &mut Vec<WtoComponent>) -> u32 {
        self.push(vertex);
        self.num += 1;
        self.dfn.insert(vertex, self.num);
        let mut head = self.num;
        let mut is_loop = false;

        for succ in self.cfg.successors(vertex) {
            let min;
            if self.dfn[&succ] == 0 {
                min = self.visit(succ, partition);
            } else {
                min = self.dfn[&succ];
            }
            if min <= head {
                head = min;
                is_loop = true;
            }
        }
        if head == self.dfn[&vertex] {
            self.dfn.insert(vertex, std::u32::MAX);
            let mut element = self.pop().unwrap();
            if is_loop {
                while element != vertex {
                    self.dfn.insert(element, 0);
                    element = self.pop().unwrap();
                }
                self.loop_heads.push(vertex);
                partition.push(WtoComponent::Circle(self.component(vertex)));
            } else {
                partition.push(WtoComponent::Vertex(WtoVertex::new(vertex)));
            }
        }
        head
    }

    fn push(&mut self, bb: BasicBlock) {
        self.stack.push(bb);
    }

    fn pop(&mut self) -> Option<BasicBlock> {
        self.stack.pop()
    }
}

/// Represents the nesting level of a node in CFG
/// E.g. for a wto: 0 ( 1 3 4 5 ) 2 6, the nesting of 3 4 5 is {1}, while the nesting of 0 1 2 6 is empty
/// Note that the head of a circle is different from its body
#[derive(Clone, PartialEq)]
pub struct WtoNesting {
    wto_nesting: Vec<WtoVertex>,
}

impl Debug for WtoNesting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = String::from("[");
        if !self.wto_nesting.is_empty() {
            for comp in &self.wto_nesting {
                r.push_str(format!("{:?},", comp).as_str());
            }
            r.pop(); // Remove the last ','
        }
        r.push(']');
        write!(f, "{}", r)
    }
}

impl WtoNesting {
    fn new() -> Self {
        Self {
            wto_nesting: Vec::new(),
        }
    }

    fn push(&mut self, vertex: WtoVertex) {
        self.wto_nesting.push(vertex);
    }

    fn pop(&mut self) {
        self.wto_nesting.pop();
    }
}

/// The trait for visitors who want to iterator over a wto
pub trait WtoVisitor {
    fn visit_vertex(&mut self, vertex: &WtoVertex);
    fn visit_circle(&mut self, circle: &WtoCircle);
    fn visit_component(&mut self, comp: &WtoComponent) {
        match comp {
            WtoComponent::Vertex(wto_vertex) => self.visit_vertex(&wto_vertex),
            WtoComponent::Circle(wto_circle) => self.visit_circle(&wto_circle),
        }
    }
}

/// Used to build a map from vertex to it corresponding nesting
struct WtoNestingIterator {
    wto_nesting: WtoNesting,
    wto_nesting_map: HashMap<WtoVertex, WtoNesting>,
}

impl Debug for WtoNestingIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r = String::from("{");
        for (v, n) in &self.wto_nesting_map {
            r.push_str(format!("{:?}: {:?},", v, n).as_str());
        }
        r.pop(); // Remove the last ','
        r.push('}');
        write!(f, "{}", r)
    }
}

impl WtoVisitor for WtoNestingIterator {
    fn visit_vertex(&mut self, vertex: &WtoVertex) {
        self.wto_nesting_map
            .insert(*vertex, self.wto_nesting.clone());
    }

    fn visit_circle(&mut self, circle: &WtoCircle) {
        let head = circle.head();
        self.wto_nesting_map.insert(head, self.wto_nesting.clone());
        self.wto_nesting.push(head);
        for comp in circle {
            self.visit_component(&comp);
        }
        self.wto_nesting.pop();
    }
}

impl WtoNestingIterator {
    fn new() -> Self {
        Self {
            wto_nesting: WtoNesting::new(),
            wto_nesting_map: HashMap::new(),
        }
    }

    fn get_wto_nesting_map(&self) -> HashMap<WtoVertex, WtoNesting> {
        self.wto_nesting_map.clone()
    }

    fn visit(&mut self, wto: &Wto) {
        for comp in wto.components() {
            self.visit_component(&comp);
        }
    }
}
