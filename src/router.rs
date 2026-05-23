fn find_route(path: &str, routes: &[RouteConfig]) -> Option<&RouteConfig> {

    // 1. exact match:
    if let Some(route) = routes.iter().find(|route| route.path == path) {
        return Some(route);
    }

    // 2. longest prefix match: 
    routes.iter()
        .filter(|route| {
            path.starts_with(route.path_as_str()) && path.as_bytes().get(route.path.len()) == Some(&b'/')
        })
        .max_by_key(|route| route.path.len())
}

/* ====================================================================================================================
NOTES:

- get(route.path.len()) == Some(&b'/'):
    if the path is "/api" this checks the last character right after the prefix for the current route:
        if it's '/' -> valid sub-route (/api/users)
        if it's anything else -> invalid match (/apix)
    so first case passes the filtering and second one fails


    
*/