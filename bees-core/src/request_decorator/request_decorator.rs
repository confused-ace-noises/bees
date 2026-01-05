use crate::{Sealed, net::{client::Handler, request::Request}};

pub trait RequestDecorator<E: Send, G: Send>: Send + Sync {
    fn decorate<'a>(&self, f: Handler<'a, E>) -> Handler<'a, G>
    where
        E: 'a,
        G: 'a;
}



#[allow(private_bounds)]
pub trait Decorate<'a, E: Send + 'a, G: Send + 'a>: Sealed {
    type Output;

    fn decorate<T: RequestDecorator<E, G> + 'a + ?Sized>(self, decorator: &'a T) -> Self::Output;    
}

impl<'a, E: std::marker::Send + 'a> Sealed for Handler<'a, E>{}
impl Sealed for Request{}


impl<'a, E: std::marker::Send + 'a, G: Send + 'a> Decorate<'a, E, G> for Handler<'a, E> {
    type Output = Handler<'a, G>;

    fn decorate<T: RequestDecorator<E, G> + 'a + ?Sized>(self, decorator: &'a T) -> Self::Output {
        decorator.decorate(self)
    }
}