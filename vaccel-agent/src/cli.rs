use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "vAccel Agent", about = "A vAccel agent that can respond to gRPC requests for acceleration requests")]
pub struct VaccelAgentCli {
    #[structopt(short = "a", long = "server-address")]
    pub uri: String
}
