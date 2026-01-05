use crate::net::client::Handler;

use super::request_decorator::RequestDecorator;

type MultipleDecoratorFunc<S, F> = Box<dyn (for<'a> Fn(Handler<'a, S>) -> Handler<'a, F>) + Send + Sync>;

pub struct MultipleDecorators<S, F> 
where
    S: Send + 'static,
    F: Send + 'static,
{
    func: MultipleDecoratorFunc<S, F>,
}

impl<E, G> MultipleDecorators<E, G> 
where
    E: Send + 'static,
    G: Send + 'static,
{
    pub fn new<RD>(request_decorator: RD) -> Self 
    where 
        RD: RequestDecorator<E, G> + 'static,
    {
        let func: MultipleDecoratorFunc<E, G> = Box::new(move |handler| request_decorator.decorate(handler));
        MultipleDecorators { func }
    }

    pub fn push<S, RD>(self, request_decorator: RD) -> MultipleDecorators<E, S> 
    where 
        S: Send,
        RD: RequestDecorator<G, S> + 'static,
    {
        let func: MultipleDecoratorFunc<E, S> = Box::new(move |handler| request_decorator.decorate((self.func)(handler)));
        MultipleDecorators { func }
    }
}

impl<E, G> RequestDecorator<E, G> for MultipleDecorators<E, G>
where
    E: Send + 'static,
    G: Send + 'static,
{
    fn decorate<'a>(self: &MultipleDecorators<E, G>, f: Handler<'a, E>) -> Handler<'a, G>
    where
        E: 'a,
        G: 'a,
    {
        (self.func)(f)
    }
}