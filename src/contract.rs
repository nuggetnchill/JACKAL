// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};

// use std::ptr::null;

use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
     StdResult, Storage
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};

use crate::state::{config, State};
use crate::backend::{query_file, query_folder_contents, try_create_folder, try_create_file, try_init, load_readonly_folder, load_readonly_file };

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {

    let ha = deps.api.human_address(&deps.api.canonical_address(&env.message.sender)?)?;

    let state = State {
        owner: ha.clone(),
    };

    config(&mut deps.storage).save(&state)?;
       
    debug_print!("Contract was initialized by {}", env.message.sender);
    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::InitAddress { seed_phrase } => try_init(deps, env, seed_phrase),
        HandleMsg::CreateFile { name, contents, path } => try_create_file(deps, env, name, contents, path),
        HandleMsg::CreateFolder { name, path } => try_create_folder(deps, env, name, path),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFile { path, address } => to_binary(&query_file(deps, address, path)?),
        QueryMsg::GetFolderContents { path, address } => to_binary(&query_folder_contents(deps, address, path)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary};
    use std::fs::read_to_string;
    use crate::backend::{make_file};
    use crate::msg::{FolderContentsResponse, FileResponse};


    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

    }

    #[test]
    fn init_test() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::InitAddress { seed_phrase: String::from("JACKAL IS ALIVE")};
        let _res = handle(&mut deps, env, msg).unwrap();
    }

    #[test]
    fn make_file_test() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::InitAddress { seed_phrase: String::from("JACKAL IS ALIVE")};
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test.txt"), contents: String::from("Hello World!"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();
    }

    #[test]
    fn make_folder_test() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::InitAddress { seed_phrase: String::from("JACKAL IS ALIVE")};
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFolder { name: String::from("new_folder"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test.txt"), contents: String::from("Hello World!"), path: String::from("/new_folder/") };
        let _res = handle(&mut deps, env, msg).unwrap();
    }

    #[test]
    fn get_file_test() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::InitAddress { seed_phrase: String::from("JACKAL IS ALIVE")};
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test.txt"), contents: String::from("Hello World!"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let res = query(&deps, QueryMsg::GetFile { address: String::from("anyone"), path: String::from("/test.txt") }).unwrap();
        let value: FileResponse = from_binary(&res).unwrap();
        assert_eq!(make_file("test.txt", "anyone", "Hello World!"), value.file);
    }

    #[test]
    fn get_folder_test() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::InitAddress { seed_phrase: String::from("JACKAL IS ALIVE")};
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFolder { name: String::from("new_folder"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test.txt"), contents: String::from("Hello World!"), path: String::from("/new_folder/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test2.txt"), contents: String::from("Hello World!"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let res = query(&deps, QueryMsg::GetFolderContents { address: String::from("anyone"), path: String::from("/") }).unwrap();
        let value: FolderContentsResponse = from_binary(&res).unwrap();
        assert_eq!(value.files, vec!["anyone/test2.txt"]);
        assert_eq!(value.folders, vec!["anyone/new_folder/"]);

        let res = query(&deps, QueryMsg::GetFolderContents { address: String::from("anyone"), path: String::from("/new_folder/") }).unwrap();
        let value: FolderContentsResponse = from_binary(&res).unwrap();
        assert_eq!(value.files, vec!["anyone/new_folder/test.txt"]);
        assert_eq!(value.folders, Vec::<String>::new());
    }

    #[test]
    fn big_files_test() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::InitAddress { seed_phrase: String::from("JACKAL IS ALIVE")};
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFolder { name: String::from("new_folder"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test.txt"), contents: String::from("Hello World!"), path: String::from("/new_folder/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test2.txt"), contents: String::from("Hello World!"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let fcont : String = read_to_string("eth.txt").unwrap();


        for i in 0..100 {
            let mut nm: String = i.to_string();
            nm.push_str(".png");

            let env = mock_env("anyone", &coins(2, "token"));
            let msg = HandleMsg::CreateFile { name: String::from(nm), contents: fcont.clone(), path: String::from("/") };
            let _res = handle(&mut deps, env, msg).unwrap();
        }
        

        let res = query(&deps, QueryMsg::GetFile { address: String::from("anyone"), path: String::from("/99.png") }).unwrap();
        let value: FileResponse = from_binary(&res).unwrap();
        assert_eq!(value.file.get_contents(), fcont.clone());



        let res = query(&deps, QueryMsg::GetFolderContents { address: String::from("anyone"), path: String::from("/") }).unwrap();
        let value: FolderContentsResponse = from_binary(&res).unwrap();

        println!("{:?}", value.files);


        assert_eq!(value.folders, vec!["anyone/new_folder/"]);

        let res = query(&deps, QueryMsg::GetFolderContents { address: String::from("anyone"), path: String::from("/new_folder/") }).unwrap();
        let value: FolderContentsResponse = from_binary(&res).unwrap();
        assert_eq!(value.files, vec!["anyone/new_folder/test.txt"]);
        assert_eq!(value.folders, Vec::<String>::new());
    }

    #[test]
    fn duplicated_folder_test() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::InitAddress { seed_phrase: String::from("JACKAL IS ALIVE")};
        let _res = handle(&mut deps, env, msg).unwrap();

        // create original copy
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFolder { name: String::from("test_folder"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test_file_one.txt"), contents: String::from("Hello Hello!!!"), path: String::from("/test_folder/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        // create duplicated copy
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFolder { name: String::from("test_folder1"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test_file_two.txt"), contents: String::from("Sup dude!!!"), path: String::from("/test_folder/") };
        let _res = handle(&mut deps, env, msg).unwrap();
        
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test_file_three.txt"), contents: String::from("Hello World!"), path: String::from("/test_folder/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let res = query(&deps, QueryMsg::GetFolderContents { address: String::from("anyone"), path: String::from("/") }).unwrap();
        let value: FolderContentsResponse = from_binary(&res).unwrap();
        println!("Querying /");
        println!("Folders: {:?}", value.folders);
        println!("Files: {:?}", value.files);

        let res = query(&deps, QueryMsg::GetFolderContents { address: String::from("anyone"), path: String::from("/test_folder/") }).unwrap();
        let value: FolderContentsResponse = from_binary(&res).unwrap();
        println!("Querying /test_folder/");
        println!("Folders: {:?}", value.folders);
        println!("Files: {:?}", value.files);

        let res = query(&deps, QueryMsg::GetFile { address: String::from("anyone"), path: String::from("/test_folder/test_file_one.txt") }).unwrap();
        let value: FileResponse = from_binary(&res).unwrap();
        println!("GetFile test_file_one: {:?}", value);

    }

    #[test]
    fn duplicated_file_test() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::InitAddress { seed_phrase: String::from("JACKAL IS ALIVE")};
        let _res = handle(&mut deps, env, msg).unwrap();

        // create original copy
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFolder { name: String::from("nice_folder"), path: String::from("/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test_file_one1.txt"), contents: String::from("Hello Hello!!!"), path: String::from("/nice_folder/") };
        let _res = handle(&mut deps, env, msg).unwrap();

        // create duplicated copy
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::CreateFile { name: String::from("test_file_one.txt"), contents: String::from("Sup dude!!!"), path: String::from("/nice_folder/") };
        let _res = handle(&mut deps, env, msg).unwrap();
        

        let res = query(&deps, QueryMsg::GetFolderContents { address: String::from("anyone"), path: String::from("/") }).unwrap();
        let value: FolderContentsResponse = from_binary(&res).unwrap();
        println!("Querying /");
        println!("Folders: {:?}", value.folders);
        println!("Files: {:?}", value.files);

        let res = query(&deps, QueryMsg::GetFolderContents { address: String::from("anyone"), path: String::from("/nice_folder/") }).unwrap();
        let value: FolderContentsResponse = from_binary(&res).unwrap();
        println!("Querying /nice_folder/");
        println!("Folders: {:?}", value.folders);
        println!("Files: {:?}", value.files);

        let res = query(&deps, QueryMsg::GetFile { address: String::from("anyone"), path: String::from("/nice_folder/test_file_one.txt") }).unwrap();
        let value: FileResponse = from_binary(&res).unwrap();
        println!("GetFile test_file_one: {:?}", value);

    }
}
