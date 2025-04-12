use std::sync::Arc;
use axum::{
    middleware,
    routing::{get, post,delete,put},
    Router,
};

use crate::handler::auth::auth;
use crate::handler::short_link::{
    SaveShortLink,
    UpdateShortLink,
    GetShortLinkList,
    GetShortLinkByID,
    DelShortLinkByID,
};

use crate::handler::director_301::{
    redirect_short_link,
};

use crate::initialize::app_state::AppState;
use crate::handler::user::{
    get_me_handler,
    health_checker_handler,
    login_user_handler,
    logout_handler,
    refresh_access_token_handler,
    register_user_handler
};



const API_PREFIX: &str = "/api";
pub fn create_router(app_state: Arc<AppState>) -> Router {
    // 健康检查路由
    let health_router = Router::new()
        .route("/healthchecker", get(health_checker_handler));

    //短链跳转301
    let shortLink_router = Router::new()
        .route("/shortLink/:code",get(redirect_short_link));

    // 认证路由组
    let auth_router = Router::new()
        .route("/auth/register", post(register_user_handler))
        .route("/auth/login", post(login_user_handler))
        .route("/auth/refresh", get(refresh_access_token_handler));

    // 需要认证的路由组
    let protected_auth_router = Router::new()
        .route("/auth/logout", get(logout_handler));

    let protected_user_router = Router::new()
        .route("/users/me", get(get_me_handler));

    //短链路有组///批量导入短连txt
    let protected_shortlink_router = Router::new()
        .route("/shortLink/createShortLink", post(SaveShortLink))
        .route("/shortLink/delShortLinkByID/:id",delete(DelShortLinkByID))
        .route("/shortLink/shortLinkList",get(GetShortLinkList))
        .route("/shortLink/updateShortLink/:id",put(UpdateShortLink))
        .route("/shortLink/getShortLinkByID/:id",get(GetShortLinkByID));

    //浏览器访问路由组
    let protected_browser_router = Router::new()
        .route("/shortLink/shortLink_list",get(SaveShortLink))
        .route("/shortLink/update_shortLink:id",put(SaveShortLink));

    //ip地址统计放访问路由组
    let protected_ip_router = Router::new()
        .route("/shortLink/shortLink_list",get(SaveShortLink))
        .route("/shortLink/update_shortLink:id",put(SaveShortLink));


    // 合并所有需要认证的路由
    let protected_routes = Router::new()
        .merge(protected_auth_router)
        .merge(protected_user_router)
        .merge(protected_shortlink_router)
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth));


    Router::new()
        .nest(API_PREFIX,
              Router::new()
                  .merge(health_router)
                  .merge(auth_router)
                  .merge(shortLink_router)
                  .merge(protected_routes)
        )
        .with_state(app_state)
}
