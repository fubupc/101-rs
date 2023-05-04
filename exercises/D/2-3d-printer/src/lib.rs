use std::marker::PhantomData;

use rand::Rng;

pub struct Printer3D<S> {
    _marker: PhantomData<S>,
}

/* States */

/// The 3D printer encountered an error and needs resetting
pub enum ErrorState {}
/// The 3D printer is waiting for a job
pub enum IdleState {}
/// The 3D printer is currently printing
pub enum PrintingState {}
/// The 3D printed product is ready
pub enum ProductReadyState {}

impl Printer3D<IdleState> {
    fn new() -> Printer3D<IdleState> {
        Printer3D {
            _marker: PhantomData,
        }
    }
}

impl Printer3D<IdleState> {
    fn start(self) -> Printer3D<PrintingState> {
        println!("start");
        Printer3D {
            _marker: PhantomData,
        }
    }
}

impl Printer3D<PrintingState> {
    fn out_of_filament(self) -> Printer3D<ErrorState> {
        println!("out of filament");
        Printer3D {
            _marker: PhantomData,
        }
    }

    fn product_ready(self) -> Printer3D<ProductReadyState> {
        println!("product ready");
        Printer3D {
            _marker: PhantomData,
        }
    }
}

impl Printer3D<ProductReadyState> {
    fn product_retrieved() -> Printer3D<IdleState> {
        println!("product retrieved");
        Printer3D {
            _marker: PhantomData,
        }
    }
}

/// Check if we're out of filament
fn out_of_filament() -> bool {
    let rand: usize = rand::thread_rng().gen_range(0..100);
    rand > 95
}
