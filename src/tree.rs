use std::collections::HashMap;
use crate::method::Method;

/// A node in radix tree
#[derive(Debug, Clone)]
pub struct Node<V> {
    path: String,
    /// The list of first letters of static path child nodes
    static_indices: Vec<char>,
    /// The list of static path child nodes
    static_child: Vec<Option<Self>>,
    /// The path parameter child node
    param_child: Option<Box<Self>>,
    /// The * wildcard child node
    star_child: Option<Box<Self>>,
    /// If this node is the end of the URL path, then call the handler.
    leaf_handler: HashMap<Method, V>,
    /// The names of the parameters
    leaf_param_names: Option<Vec<String>>,
}

impl<V: Clone> Node<V> {
    pub fn new() -> Self {
        Self {
            path: "/".to_string(),
            ..Default::default()
        }
    }

    pub fn insert(&mut self, method: Method, path: &str, value: V) {
        self.insert_path(&method, strip_start_slash(path.to_string()), value, None);
    }

    pub fn remove(&mut self, path: &str) {
        self.remove_path(strip_start_slash(path.to_string()));
    }

    pub fn update(&mut self, method: Method, path: &str, value: V) {
        self.update_path(&method, strip_start_slash(path.to_string()), value);
    }

    /// Returns Some(SearchResult<V>), if successful. Otherwise returns None.
    pub fn search(&self, method: Method, path: &str) -> Option<SearchResult<V>> {
        let match_result = self.internal_search(&method, strip_start_slash(path.to_string()));

        match_result.map(|v| SearchResult {
            value: v.value.clone().unwrap(),
            params: v.from_params(),
        })
    }

    fn insert_path(&mut self, method: &Method, path: String, value: V, param_names: Option<Vec<String>>) {
        if path.is_empty() {
            // Assign a value to self.leaf_param_names
            if let Some(ref param_names) = param_names {
                // Make sure the current path parameters are the same as the old ones.
                // When they aren't, we have a ambiguous path.
                if let Some(ref leaf_param_names) = self.leaf_param_names {
                    if param_names.len() != leaf_param_names.len() {
                        // This should never happen.
                        panic!("Reached leaf node with differing the number of path parameters. Please report this as a bug.");
                    }

                    // Ambiguous path, such as /hello/$a and /hello/$b.
                    if param_names != leaf_param_names {
                        panic!("Path parameters {:?} are ambiguous with {:?}.", leaf_param_names, param_names);
                    }
                } else {
                    self.leaf_param_names = Some(param_names.clone());
                }
            }

            self.set_handler(method.clone(), value);
            return;
        }

        let first_char = path.chars().next().unwrap();
        let next_slash = path.chars().position(|c| c == '/');

        // Token is the path of the current node
        let (token, token_end) = if first_char == '/' {
            ("/".to_string(), Some(1))
        } else if next_slash.is_none() {
            (path.clone(), Some(path.len()))
        } else {
            (path.chars().take(next_slash.unwrap_or_default()).collect(), next_slash)
        };

        let remaining_path = path.chars().skip(token_end.unwrap_or_default()).collect();

        if first_char == '$' { // Handle path parameters
            // Token is the path of the current node and also the parameter name.
            let token = token[1..].to_string();

            if self.param_child.is_none() {
                self.param_child = Some(Box::new(Node {
                    path: token.clone(),
                    ..Default::default()
                }));
            }

            let param_names = param_names.map(|mut v| {
                v.push(token.clone());
                v
            }).or_else(|| Some(vec![token]));

            self.param_child.as_mut().unwrap().insert_path(method, remaining_path, value, param_names);
        } else if first_char == '*' { // Handle the * wildcard
            if path != "*" {
                panic!("Other characters were found after *");
            }

            if self.star_child.is_none() {
                self.star_child = Some(Box::new(Node {
                    ..Default::default()
                }));
            }

            self.star_child.as_mut().map(|node| {
                node.set_handler(method.clone(), value);
                node.leaf_param_names = param_names;
            });
        } else { // Handle static path
            // Do we have an existing node that starts with the same letter?
            for (i, c) in self.static_indices.clone().iter().enumerate() {
                if first_char == *c {
                    // Yes. Split it based on the existing node.
                    let len = self.split_common_prefix(i, token.clone());

                    self.static_child.get_mut(i).unwrap().as_mut().map(|v| {
                        v.insert_path(method, path[len..].to_string(), value.clone(), param_names);
                    });

                    return;
                }
            }

            // No existing node starting with the letter, so create it.
            let mut child_node = Self {
                path: token,
                ..Default::default()
            };

            self.static_indices.push(first_char);
            child_node.insert_path(method, remaining_path, value, param_names);
            self.static_child.push(Some(child_node));
        }
    }

    fn remove_path(&mut self, path: String) {
        if path.is_empty() {
            self.leaf_handler.clear();
            self.leaf_param_names = None;
            return;
        }

        let path_len = path.len();

        // First see if this matches a static path
        let first_char = path.chars().next().unwrap();
        for (i, c) in self.static_indices.iter().enumerate() {
            if first_char == *c {
                let static_child = self.static_child[i].as_mut().unwrap();
                let static_child_path_len = static_child.path.len();
                if path_len >= static_child_path_len && path.starts_with(static_child.path.as_str()) {
                    let next_path = path.chars().skip(static_child_path_len).collect();
                    static_child.remove_path(next_path);

                    if static_child.leaf_handler.is_empty()
                        && static_child.static_child.is_empty()
                        && static_child.param_child.is_none()
                        && static_child.star_child.is_none() { // Remove static child node
                        self.static_child.remove(i);
                        self.static_indices.remove(i);
                    } else { // Merge nodes
                        if static_child.leaf_handler.is_empty()
                            && static_child.static_child.len() == 1
                            && static_child.param_child.is_none()
                            && static_child.path.ne("/") {
                            let static_child_child = static_child.static_child[0].as_mut().unwrap();
                            if static_child_child.path.ne("/") {
                                let static_child_path = static_child.path.clone();
                                static_child_child.path = static_child_path + static_child_child.path.as_str();
                                self.static_child[i] = Some(static_child_child.clone());
                            }
                        }
                    }

                    return;
                }

                break;
            }
        }

        // Didn't find a static path, so check for a path parameter.
        if path.starts_with("$") {
            if let Some(ref mut param_child) = self.param_child {
                let next_slash = path.chars().position(|c| c == '/').unwrap_or(path_len);
                let next_path = path.chars().skip(next_slash).collect();
                param_child.remove_path(next_path);

                // Remove param child node
                if param_child.leaf_handler.is_empty() && param_child.static_child.is_empty() {
                    self.param_child = None;
                }
                return;
            }
        }

        // Finally check for a wildcard *
        if path.starts_with("*") {
            if self.star_child.is_some() {
                // Remove wildcard * child node
                self.star_child = None;
            }
        }
    }

    fn update_path(&mut self, method: &Method, path: String, value: V) {
        if path.is_empty() {
            self.update_handler(method.clone(), value);
            return;
        }

        let path_len = path.len();

        // First see if this matches a static path
        let first_char = path.chars().next().unwrap();
        for (i, c) in self.static_indices.iter().enumerate() {
            if first_char == *c {
                let static_child = self.static_child[i].as_mut().unwrap();
                let static_child_path_len = static_child.path.len();
                if path_len >= static_child_path_len && path.starts_with(static_child.path.as_str()) {
                    let next_path = path.chars().skip(static_child_path_len).collect();
                    static_child.update_path(method, next_path, value);
                    return;
                }
                break;
            }
        }

        // Didn't find a static path, so check for a path parameter.
        if path.starts_with("$") {
            if let Some(ref mut param_child) = self.param_child {
                let next_slash = path.chars().position(|c| c == '/').unwrap_or(path_len);
                let next_path: String = path.chars().skip(next_slash).collect();
                param_child.update_path(method, next_path, value);
                return;
            }
        }

        // Finally check for a wildcard *
        if path.starts_with("*") {
            if let Some(ref mut star_child) = self.star_child {
                star_child.update_handler(method.clone(), value);
            }
        }
    }

    fn internal_search(&self, method: &Method, path: String) -> Option<MatchResult<V>> {
        if path.is_empty() {
            if self.leaf_handler.is_empty() {
                return None;
            }

            return Some(MatchResult {
                value: self.leaf_handler.get(method).cloned(),
                param_names: self.leaf_param_names.clone().unwrap_or_default(),
                param_values: Vec::new(),
            });
        }

        let path_len = path.len();

        // First see if this matches a static path
        let first_char = path.chars().next().unwrap();
        let mut found = None;
        for (i, c) in self.static_indices.iter().enumerate() {
            if first_char == *c {
                let static_child = self.static_child[i].as_ref().unwrap();
                let static_child_path_len = static_child.path.len();
                if path_len >= static_child_path_len && path.starts_with(static_child.path.as_str()) {
                    let next_path = path.chars().skip(static_child_path_len).collect();
                    found = static_child.internal_search(method, next_path);
                }
                break;
            }
        }

        // If we find a node and it has a valid handler, then return here.
        if found.as_ref().filter(|v| v.value.is_some()).is_some() {
            return found;
        }

        // Didn't find a static path, so check for a path parameter.
        if let Some(ref param_child) = self.param_child {
            let next_slash = path.chars().position(|c| c == '/').unwrap_or(path_len);
            // Value is the parameter value
            let value: String = path.chars().take(next_slash).collect();
            let next_path: String = path.chars().skip(next_slash).collect();

            if !value.is_empty() { // Don't match on empty value
                let match_result = param_child.internal_search(method, next_path);

                if match_result.as_ref().filter(|v| v.value.is_some()).is_some() {
                    // Handle the values of the path parameters
                    if let Some(mut match_result) = match_result {
                        let mut param_values = vec![value];
                        param_values.append(&mut match_result.param_values.to_vec());
                        match_result.param_values = param_values;

                        return Some(match_result);
                    }
                }
            }
        }

        // Finally check for a wildcard *
        if let Some(ref star_child) = self.star_child {
            let value = star_child.leaf_handler.get(method);

            if value.is_some() {
                return Some(MatchResult {
                    value: value.cloned(),
                    param_names: star_child.leaf_param_names.clone().unwrap_or_default(),
                    param_values: Vec::new(),
                });
            }
        }

        None
    }

    /// Returns the length of the common prefix
    fn split_common_prefix(&mut self, existing_node_index: usize, path: String) -> usize {
        let child_node = self.static_child.get(existing_node_index).unwrap().as_ref().unwrap();

        if path.starts_with(child_node.path.as_str()) {
            // No split needs to be done. Rather, the new path shares the entire
            // prefix with the existing node, so the new node is just a child of
            // the existing node. Or the new path is the same as the existing path,
            // which means that we just move on to the next token.
            let len = child_node.path.len();
            return len;
        }

        let len = path.chars().zip(child_node.path.chars()).take_while(|(l, r)| l == r).count();
        let common_prefix = path[..len].to_string();
        let child_path = child_node.path.chars().skip(len).collect::<String>();
        let new_node = Self {
            path: common_prefix,
            static_indices: vec![child_path.chars().next().unwrap()],
            ..Default::default()
        };

        // Use new_node to replace child_node
        let mut old_node = self.static_child[existing_node_index].replace(new_node);
        old_node.as_mut().map(|v| {
            v.path = child_path;
        });

        // Old_node as a child node
        self.static_child.get_mut(existing_node_index).unwrap().as_mut().map(|v| {
            v.static_child.push(old_node);
        });

        len
    }

    fn set_handler(&mut self, method: Method, value: V) {
        if self.leaf_handler.contains_key(&method) {
            panic!("A method of a path only appear once.");
        }

        self.leaf_handler.insert(method, value);
    }

    fn update_handler(&mut self, method: Method, value: V) {
        if !self.leaf_handler.contains_key(&method) {
            panic!("This method does not exist for this path.");
        }

        self.leaf_handler.insert(method, value);
    }
}

impl<V> Default for Node<V> {
    fn default() -> Self {
        Self {
            path: "".to_string(),
            static_indices: Vec::new(),
            static_child: Vec::new(),
            param_child: None,
            star_child: None,
            leaf_handler: HashMap::new(),
            leaf_param_names: None,
        }
    }
}

/// The response returned when getting the value for a specific path.
#[derive(Debug)]
pub struct SearchResult<V> {
    value: V,
    /// The path parameters
    params: Vec<Param>,
}

impl<V> SearchResult<V> {
    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
}

/// Param is a single path parameter, consisting of a name and a value.
#[derive(Debug)]
pub struct Param {
    name: String,
    value: String,
}

impl Param {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

/// The response returned when getting the value for a specific path.
struct MatchResult<V> {
    value: Option<V>,
    /// The names of the path parameters
    param_names: Vec<String>,
    /// The values of the path parameters
    param_values: Vec<String>,
}

impl<V> MatchResult<V> {
    fn from_params(&self) -> Vec<Param> {
        let mut params = Vec::new();
        for (index, name) in self.param_names.iter().enumerate() {
            let value = self.param_values.get(index).unwrap();
            params.push(Param::new(name.clone(), value.clone()));
        }

        params
    }
}

fn strip_start_slash(path: String) -> String {
    if path.starts_with("/") {
        path.strip_prefix("/").unwrap().to_string()
    } else {
        path
    }
}
