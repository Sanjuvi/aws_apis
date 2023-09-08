use aws_config::SdkConfig;
use aws_sdk_memorydb::{
    types::{Endpoint, Snapshot,InputAuthenticationType, AuthenticationMode,Authentication},
    Client as MemDbClient,
};
use colored::Colorize;
pub struct MemDbOps {
    config: SdkConfig,
}
impl MemDbOps {
    pub fn build(config: SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }

    //node type =The compute and memory capacity of the nodes in the cluster
    //possible node values = vec!["db.t4g.small","db.r6g.large","db.r6g.xlarge,"db.r6g.2xlarge]
    //
    pub async fn create_memdb_cluster(
        &self,
        node_type: &str,
        cluster_name: &str,
        access_control_list_name: &str,
    ) {
        let config = self.get_config();
        let client = MemDbClient::new(config);
        client.create_cluster()
                    .acl_name(access_control_list_name)
                    .cluster_name(cluster_name)
                    .node_type(node_type)
                    .send()
                    .await
                    .map(|output|{
                        let colored_msg = format!("The cluster with the name {cluster_name} has been created, and the process of starting it is now underway").green().bold();
                        println!("{colored_msg}");
                        let status =if let Some(cluster)= output.cluster{
                            if let Some(status_) = cluster.status{
                                 Some(status_)
                            }else {
                                None
                            }
                        }else{
                            None
                        };
                        if let Some(status) = status {
                            let colored_status = status.green().bold();
                            println!("The Present State of the MemDb Cluster: {colored_status}\n");
                          }
                       
                    })
                    .expect("Error while creating memory db cluster");
    }

    pub async fn create_memdb_user(&self,username:&str,acl_name:&str,
     authenticate_type :&str,authenticate_passwords:&str
    ) {
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let authenticate_type = match authenticate_type {
            "iam" | "Iam" =>InputAuthenticationType::Iam,
            "password" | "Password" => InputAuthenticationType::Password,
            _ => panic!("Wrong authentication types: {}\n",authenticate_type)
        };
        let get_passwords = authenticate_passwords.split_whitespace()
                        .map(|str|str.to_string())
                        .collect::<Vec<String>>();

        let build_auth_type = AuthenticationMode::builder()
                           .set_type(Some(authenticate_type))
                           .set_passwords(Some(get_passwords))
                           .build();

        let create_user_output = client.create_user()
                   .set_access_string(Some(acl_name.into()))
                   .set_user_name(Some(username.into()))
                   .set_authentication_mode(Some(build_auth_type))
                   .send().await
                   .expect("Error while creating user in MemoryDB\n");
       let user  = create_user_output.user;
       if let Some(user) = user {
          if let Some(status) = user.status{
            let colored_status = status.green().bold();
            println!("The status of user: {}\n",colored_status);
          }else {
              println!("The satus of user: None\n")
          }
       }
    
    }

    pub async fn describe_memdb_cluster(&self, cluster_name: &str) -> Vec<MemDbClusterInfo> {
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let cluster_info = client
            .describe_clusters()
            .cluster_name(cluster_name)
            .send()
            .await
            .expect("Error while Describing the memdb clusters");
        let cluster_info = cluster_info.clusters;

        let mut vec_of_memdbclusterinfo = Vec::new();

        if let Some(vec_of_cluster) = cluster_info {
            vec_of_cluster.into_iter().for_each(|cluster_info| {
                let cluster_end_point = cluster_info.cluster_endpoint;
                let acl_name = cluster_info.acl_name;
                let status = cluster_info.status;
                let engine_version = cluster_info.engine_version;
                let memdbinfo =
                    MemDbClusterInfo::build_memdbclusterinfo(cluster_end_point, acl_name, status,engine_version);
                vec_of_memdbclusterinfo.push(memdbinfo);
            });
        }
        vec_of_memdbclusterinfo
    }

///Only returns the single insatnce of user instead of vector of user.
    pub async fn describe_memdb_user(&self,username:&str)->Vec<MemDBUser>{
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let output = client.describe_users()
                    .user_name(username)
                    .send().await
                    .expect("Error while describing memdb cluster");
        let user = output.users; 
        let mut single_user_info = Vec::new();
        if let Some(mut vec_of_users) =  user{
            let single_user_ = vec_of_users.drain(..1);
            single_user_.into_iter()
            .for_each(|user|{
             let user_name = user.name; 
             let status = user.status; 
             let access_string = user.access_string; 
             let authentication = user.authentication;
            single_user_info.push(MemDBUser::build_memdbuser_info(user_name, status, access_string, authentication));
            });
        }      

        single_user_info
        
    }
    pub async fn describe_snapshots(&self, cluster_name: &str) -> Vec<Snapshot> {
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let snapshots = client
            .describe_snapshots()
            .cluster_name(cluster_name)
            .send()
            .await
            .expect("Error while describing snapshots of memdb");
        let mut vec_of_snapshots = Vec::new();
        let snapshots = snapshots.snapshots;

        if let Some(vec_of_snapshot) = snapshots {
            vec_of_snapshot.into_iter().for_each(|snapshot| {
                vec_of_snapshots.push(snapshot);
            })
        }
        vec_of_snapshots
    }

    pub async fn delete_memdb_cluster(&self, cluster_name: &str, final_snapshot_name: &str) {
        let config = self.get_config();
        let client = MemDbClient::new(config);

        client.delete_cluster()
                .cluster_name(cluster_name)
                .final_snapshot_name(final_snapshot_name)
                .send()
                .await
                .map(|output|{
                    println!("The MemDB cluster named {cluster_name} has initiated the cluster deletion process.");
                    let status = if let Some(cluster) = output.cluster  {
                        if let Some(status) = cluster.status{
                            Some(status)
                        }else {
                            None
                        }
              }else{
                None
              };
              if let Some(status) = status {
                let colored_status = status.green().bold();
                println!("The Present State of the MemDb Cluster: {colored_status}\n");
              }
                })
                .expect("Error while deleteing memdb cluster");
    }

    pub async fn delete_memdb_user(&self,username:&str){
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let ouput = client.delete_user()
                  .user_name(username)
                  .send().await
                  .expect("Error while deleting memdb user");
        if let Some(user) = ouput.user {
            if let (Some(status),Some(name)) = (user.status,user.name) {
                let colored_name = name.green().bold();
                let colored_status = status.green().bold();
                println!("The Name of User: {colored_name}\nThe current status: {colored_status}\n")
                
            }
        }
    }
}


#[derive(Debug)]
pub struct MemDBUser{
    user_name: Option<String>,
    status: Option<String>,
    access_string : Option<String>,
    authentication : Option<Authentication>,
}

impl MemDBUser {
    fn build_memdbuser_info(
        user_name:Option<String>,status:Option<String>,access_string:Option<String>,
        authentication:Option<Authentication>
    )->Self{
        Self {
            user_name, 
            status, 
            access_string, 
            authentication }
    }

    pub fn get_username(&self)->Option<&str>{
       self.user_name.as_deref()
    }

    pub fn print_auth_info(&self){
      if let Some(authentication) = self.authentication.as_ref() {
          if let Some(auth_type) = authentication.r#type() {
              let colored_auth_type = auth_type.as_str().green().bold();
              println!("Authentication Type: {colored_auth_type}\n");
          }
          if let Some(pass_count) = authentication.password_count{
            let colored_count = pass_count.to_string().green().bold();
            println!("Password count: {colored_count}\n");
          }
          
      }
    }

    pub fn get_acess_string(&self) -> Option<&str>{
        self.access_string.as_deref()
    }
    pub fn get_status(&self)->Option<&str>{
        self.status.as_deref()
    }
}

#[derive(Debug)]
pub struct MemDbClusterInfo {
    cluster_end_point: Option<Endpoint>,
    acl_name: Option<String>,
    status: Option<String>,
    redis_engine_version : Option<String>
}
impl MemDbClusterInfo {
    fn build_memdbclusterinfo(
        cluster_end_point: Option<Endpoint>,
        acl_name: Option<String>,
        status: Option<String>,
        redis_engine_version: Option<String>
    ) -> Self {
        Self {
            cluster_end_point,
            acl_name,
            status,
            redis_engine_version
        }
    }
    pub fn get_status(&self) -> Option<String> {
        let status = self.status.clone();
        if let Some(status) = status {
            Some(status)
        } else {
            None
        }
    }
    pub fn get_port(&self)->Option<i32>{
      let endpoint = self.cluster_end_point.as_ref();

      if let Some(endpoint) = endpoint {
          Some(endpoint.port)
      }else {
          None
      }
    }

    pub fn get_endpoint_with_port(&self) -> Option<String> {
        let status = self.get_status();
        println!("Current Status of MemDbInstance: {status:?}\n");
        let connection_url = if let Some(endpoint) = self.cluster_end_point.as_ref() {
            if let Some(database_url) = endpoint.address() {
                let mut url = database_url.to_string();
                let port = endpoint.port;
                let port_str = format!(":{port}");
                url.push_str(&port_str);
                Some(url)
            } else {
                None
            }
        } else {
            None
        };
        connection_url
    }

    pub fn get_acl_name(&self) -> Option<String> {
        let acl_name = self.acl_name.clone();
        if let Some(acl_name) = acl_name {
            Some(acl_name)
        } else {
            None
        }
    }

    pub fn get_redis_version(&self)->Option<&str>{
        self.redis_engine_version.as_deref()
    }
}
