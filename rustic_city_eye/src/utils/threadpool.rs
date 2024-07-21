use std::{
    sync::{
        mpsc::{self, channel, Receiver},
        Arc, Mutex,
    },
    thread,
};
/// El codigo que estan por ver es un poco falopa, pasa que tiene que ser una interfaz muy generalista
/// Los workers tienen un thread y un ID, se crean segun el valor que se le pase por parametro al constructor.
/// Si se ejecuta new(4), se crearan 4 workers.
/// el sender es el encargado de recibir el nuevo job a ejecutar.
#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}
#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = match receiver.lock() {
                Ok(lock) => match lock.recv() {
                    Ok(job) => job,
                    Err(_) => {
                        return;
                    }
                },
                Err(err) => {
                    println!("Failed to acquire lock: {:?}", err);
                    continue;
                }
            };

            job();
            break;
        });

        Worker { id, thread }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Esta funcion recibe una funncion F y devuelve un reciever con el resultado de la misma.
    /// el channel creado internamente es para ejecutar la funcion F y luego retornar el resultado de su ejecución
    /// Se le pide entonces al job que ejecute F y mande su resultado por el channel.
    /// Luego se le pasa ese job a un worker disponible mediante el self.sender.
    /// Es en ese momento que la funcion F se ejecuta.
    pub fn execute<F, R>(&self, f: F) -> Receiver<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = channel();
        let senderr = self.sender.clone();

        let job = Box::new(move || {
            let result = f();
            if tx.send(result).is_err() {
                // println!("Failed to send result: {:?}", err);
            }
        });

        if let Err(err) = senderr.send(job) {
            println!("Failed to send job: {:?}", err);
        }

        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threadpool() {
        let pool = ThreadPool::new(4);

        let reciever = pool.execute(|| {
            println!("Hello from the threadpool");
            42
        });

        assert_eq!(reciever.recv().unwrap(), 42);
    }
}
