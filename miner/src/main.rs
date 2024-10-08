use lib::types::Block; 
use lib::util::Saveable; 

use std::{env};
use std::process::exit;
use lib::crypto::PublicKey;
use lib::network::Message;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use std::thread;


use anyhow::{anyhow, Ok, Result};   // anyhow - idiomatic{standard way} error handling in Rust applications
use clap::Parser;   // clap- command line parser
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
    };


    

    // atomic::{AtomicBool, Ordering}:
    
    //     AtomicBool: This is an atomic type that represents a boolean value that can be safely shared between threads. It supports atomic operations, which means operations on it will not cause data races.
    //     Ordering: This is an enumeration that defines memory ordering semantics for atomic operations. Common variants include:
    //         Ordering::Relaxed: No synchronization or ordering guarantees beyond atomicity.
    //         Ordering::Acquire: Ensures that subsequent reads and writes cannot be reordered before this operation.
    //         Ordering::Release: Ensures that previous reads and writes cannot be reordered after this operation.
    //         Ordering::SeqCst: Provides the strongest guarantees, ensuring a single total order for all sequentially consistent operations.
    
    // Arc: This stands for "Atomic Reference Counted." It is a thread-safe reference-counting pointer. Arc allows multiple threads to share ownership of the same data. When the last reference to the data goes out of scope, the data is deallocated.

    // flume library provide fast, async/sync hybrid mpmc(multi producers multi consumers) channels

    #[derive(Parser)]
    #[command(author, version, about, long_about = None)]  // this attribute is used to provide metadata and configuration options for the command-line interface (CLI) that the application will expose like who is the author, and version of applicatioin

    struct Cli {

        #[arg(short, long)]
        address: String,

        #[arg(short, long)]
        public_key_file: String,
    }

struct Miner{

    public_key: PublicKey,

    stream: Mutex<TcpStream>,

    current_template: Arc<std::sync::Mutex<Option<Block>>>,

    mining: Arc<AtomicBool>,

    mined_block_sender: flume::Sender<Block>,

    mined_block_receiver: flume::Receiver<Block>,
}

// A thread-safe operation ensures that, when multiple threads execute in parallel, t
//they don’t interfere with each other or cause unintended side effects 
// like data races, deadlocks, or corrupted states.

impl Miner {

    async fn new(address: String, public_key: PublicKey) -> Result<Self> {


        let stream = TcpStream::connect(&address).await?;

        let (mined_block_sender, mined_block_receiver) = flume::unbounded();

        Ok(Self {

            public_key,

            stream: Mutex::new(stream),

            current_template: Arc::new(std::sync::Mutex::new(None,)),  // // Arc (Atomic Reference Count)

            mining: Arc::new(AtomicBool::new(false)),

            mined_block_sender,
            
            mined_block_receiver,
                   



        })


    }

    async fn run(&self) -> Result<()> {


        self.spawn_mining_thread();

        let mut template_interval = interval(Duration::from_secs(5));

        loop {
            
            let receiver_clone = self.mined_block_receiver.clone();
            

             //  wait for multiple async operation to complete
            tokio::select! {

               
                _ = template_interval.tick() => {

                    self.fetch_and_validate_template().await?;
                }

                std::result::Result::Ok(mined_block) = receiver_clone.recv_async() => {

                    self.submit_block(mined_block).await?;
                }
            }
        }



    }

    fn spawn_mining_thread(&self) -> thread::JoinHandle<()> {

        let template = self.current_template.clone(); // creates another pointer to same resource

        let mining = self.mining.clone();

        let sender = self.mined_block_sender.clone();


        thread::spawn(move || {

            if mining.load(Ordering::Relaxed) {

                if let Some(mut block) = template.lock().unwrap().clone() {

                    println!("Mining block with target: {}", block.header.target );

                

                    if block.header.mine(2_000_000) {

                        println!("Block mined: {}", block.hash());
                    

                        sender.send(block).expect("failed to send mined block");
                        
                        // Setting mining to false indicates that mining has stopped (probably 
                        // because a block was successfully mined), and no more mining should be done.
                        mining.store(false, Ordering::Relaxed);
                    }
                }

            }

            // thread::yield_now()  hints to the operating system that the current thread is willing to give up its turn on the CPU, 
            // allowing other threads to run.

            thread::yield_now();

        })

    }

    async fn fetch_and_validate_template(&self) -> Result<()> {


        if !self.mining.load(Ordering::Relaxed) {

            self.fetch_template().await?;

        } else {

            self.validate_template().await?;

        }

        Ok(())



    }

    async fn fetch_template(&self) -> Result<()> {


        println!("fetchintg new template");

        let message = Message::FetchTemplate(self.public_key.clone());

        let mut stream_lock = self.stream.lock().await;

        message.send_async(&mut *stream_lock).await?;
        
        // This explicitly drops the lock on the stream, releasing the Mutex or RwLock.
        //This is important because once the message is sent, the function no longer needs exclusive access to the stream
        drop(stream_lock);

        
        let mut stream_lock = self.stream.lock().await;
        match Message::recieve_asynce(&mut *stream_lock).await? {

            Message::Template(template) =>  {

                drop(stream_lock);
                
                println!("received new template with target: {}", template.header.target);

                *self.current_template.lock().unwrap() = Some(template);

                self.mining.store(true, Ordering::Relaxed);

                Ok(())

            }

            _ => Err(anyhow!("Unexpected message received when fetching")),                                            

        }


    }

    async fn validate_template(&self) -> Result<()> {


        if let Some(template) = self.current_template.lock().unwrap().clone() { 

            let message = Message::ValidateTemplate(template);

            let mut stream_lock = self.stream.lock().await;

            message.send_async(&mut *stream_lock).await?;

            drop(stream_lock);

            let mut stream_lock = self.stream.lock().await;

            match Message::recieve_asynce(&mut *stream_lock).await? {

                Message::TemplateValidity(valid) => {

                    drop(stream_lock);

                    if !valid {

                        println!("current template is no longer valid");

                        self.mining.store(false, Ordering::Relaxed);

                    } else {

                        println!(" current template is still valid")
                    }

                    Ok(())


                }

                _ => Err(anyhow!("Unexpected message received when validating template"))
            }

        }  else {

            Ok(())
        }


    }

    async fn submit_block(&self, block: Block) -> Result<()> {


        println!("submitting the block");

        let message = Message::SubmitTemplate(block);

        let mut stream_lock = self.stream.lock().await;

        message.send_async(&mut *stream_lock).await?;
        
        self.mining.store(false, Ordering::Relaxed);

        Ok(())

    }
}






fn usage() -> !{

    eprint!(
        "Usage: {} <address> <public_key_files>",
        env::args().next().unwrap()
    );

    exit(1);
}

// Sets Up the Tokio Runtime: The macro automatically initializes and runs a Tokio runtime. 
// This runtime is responsible for managing and executing asynchronous tasks in the program.
// Without this macro, you would need to manually create and manage the runtime.

// Allows main to Be Async: 
//In Rust, the main function is usually synchronous(running task sequentially) 
//(i.e., cannot use async). However, by adding #[tokio::main], you can write an async version of main, allowing you to use await and other asynchronous features in your program's entry point


#[tokio::main]
async fn main() -> Result<()>{
    

   

    let cli = Cli::parse();

    let public_key = PublicKey::load_from_file(&cli.public_key_file).map_err(|e| {
                                    anyhow!("Error reading public key: {}", e)
                                    })?;

    let miner = Miner::new(cli.address, public_key).await?;
    
    miner.run().await


    


}








// 1.Reactor

// The reactor is a core component in an event-driven system that waits for events (like I/O readiness) to occur and notifies the program when they do. In the context of async Rust, the reactor is responsible for monitoring I/O resources and notifying the program when those resources are ready for processing.

//     It typically runs in the background, waiting for events like network data arriving or a file being ready to read.
//     When an event occurs (e.g., a socket becomes readable), the reactor signals the executor that a task can make progress.
//     Reactors in Rust are usually provided by libraries like tokio or async-std, which handle event loops under the hood.

// Example: In tokio, the reactor might watch for a network socket to be ready for reading and signal the executor when it's ready.
// 2. Executor

// The executor is responsible for running async tasks. Once the reactor signals that some I/O operation is ready, the executor schedules the async task associated with that event to be resumed.

//     An executor manages a set of tasks that need to be run or resumed.
//     It runs the tasks in an event loop, pulling tasks off a queue, running them until they yield (i.e., await something), and then putting them back until they can continue.
//     The executor ensures that each async task is executed to the point where it needs to await some other event (like I/O readiness), and then the task is paused until the awaited event happens.

// Example: tokio::spawn is a function that runs an async task on an executor, which handles scheduling and running it until it's completed or needs to await something.
// 3. Runtime

// The runtime is essentially the combination of both the reactor and the executor, orchestrating the entire system to handle async tasks. It provides the environment in which async tasks are executed. A runtime typically includes:

//     A reactor to manage and poll for events (I/O readiness, timers, etc.).
//     An executor to manage the task queue and handle task scheduling and running.
//     Sometimes, additional features like timers, blocking operations, and signals are part of the runtime.




// n Rust, the load() 
//It allows you to safely read the value from an atomic type,
// ensuring proper memory ordering between threads.

// The load() method takes a parameter of type Ordering, which determines the memory ordering guarantees. 
//Here are the most commonly used options:

//     Ordering::Relaxed: No guarantees about memory ordering. Only the atomicity of the operation is guaranteed.
//     Ordering::Acquire: Ensures that all previous reads and writes become visible before the current operation.
//     Ordering::SeqCst: The strictest ordering, ensuring that all threads agree on the order of all sequentially consistent operations.