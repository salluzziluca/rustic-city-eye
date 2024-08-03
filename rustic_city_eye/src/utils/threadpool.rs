use std::{
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};
/// Maneja una pool de worker threads.
/// Los Workers son creados basandose en el tamaño especificado para la pool cuando se crea(en el `new`).
/// El Sender es el responsable de recibir nuevos
#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

/// La estructura `Worker` representa un único trabajador en el pool de hilos.
/// Cada Worker tiene un ID único y un hilo que ejecuta los trabajos recibidos del pool de hilos.
#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Crea una nuevo Worker con el id y el receiver dado.
    ///
    /// El id es un identificador unico para el Worker.
    /// El Receiver es responsable de recibir nuevos trabajos para ejecutar.
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
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
    /// Crea una ThreadPool nueva con el tamaño especificado.
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
    ///
    /// El channel creado internamente es para ejecutar la funcion F y luego retornar el resultado de su ejecución:
    /// se le pide entonces al job que ejecute F y mande su resultado por el channel.
    ///
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
            let _ = tx.send(result);
        });

        if let Err(err) = senderr.send(job) {
            println!("Failed to send job: {:?}", err);
        }

        rx
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    #[test]
    fn test_threadpool_multiple_jobs() {
        let pool = ThreadPool::new(4);

        let receiver1 = pool.execute(|| 1);

        let receiver2 = pool.execute(|| 2);

        assert_eq!(receiver1.recv().unwrap(), 1);
        assert_eq!(receiver2.recv().unwrap(), 2);
    }

    #[test]
    fn test_threadpool_shared_state() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        let counter1 = Arc::clone(&counter);
        let receiver1 = pool.execute(move || {
            counter1.fetch_add(1, Ordering::SeqCst);
        });

        let counter2 = Arc::clone(&counter);
        let receiver2 = pool.execute(move || {
            counter2.fetch_add(1, Ordering::SeqCst);
        });

        receiver1.recv().unwrap();
        receiver2.recv().unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_two_way_communication() {
        let pool = ThreadPool::new(2);

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let receiver1 = pool.execute(move || {
            tx1.send("Message from thread 1").unwrap();
            rx2.recv().unwrap();
        });

        let receiver2 = pool.execute(move || {
            let message = rx1.recv().unwrap();
            tx2.send(format!("Thread 2 received: {}", message)).unwrap();
        });

        receiver1.recv().unwrap();
        receiver2.recv().unwrap();

        // assert_eq!(response1,() );
    }
}
