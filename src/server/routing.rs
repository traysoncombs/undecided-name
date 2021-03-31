use warp::*;


pub fn get_routes() -> impl warp::Filter {
  let index = warp::path::end().map(|| "Index");

  let register = warp::post()
      .and(warp::path("register"))
      .map(register);

  let login = warp::post()
      .and(warp::path("login"))
      .map(login);

  let routes = index
      .or(register)
      .or(login);
  routes
}

/*
  Returns true or false depending on whether or not the user was succesfully registered.
  Password should be a hash of the actual password.
*/

fn register(username: &String, password: &String) -> String {

}

/*
  Returns auth token for user.
  Takes the hash of the password, this should be hashed by the client.
*/

fn login(username: &String, password: &String) -> String {

}



