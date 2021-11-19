use welkin_binding::{bind, impl_wrapper};

mod conversions;
pub mod io;

bind! {
    #[path = "./welkin/defs"]
    #[include(
        Unit,
        Bool,
        Pair,
        Vector,
        Word,
        Char,
        String,
        IO,
        WhelkRequest,
        Whelk,
        Ast
    )]
    pub mod w {
        #[wrapper = "IO::call.then"]
        struct IoThen;

        #[wrapper = "Size"]
        struct Size;

        enum Sized<A> {
            new {
                size: Size,
                data: A
            }
        }

        #[wrapper = "WhelkRequest::loop.initial"]
        struct WhelkRequestLoopInitialState;

        #[wrapper = "WhelkRequest::loop.continue"]
        struct WhelkRequestLoopContinuePredicate;

        #[wrapper = "WhelkRequest::loop.step"]
        struct WhelkRequestLoopStep;

        #[wrapper = "Any"]
        struct Any;

        #[indices = 1]
        type WhelkIO<A> = IO<WhelkRequest, A>;

        #[indices = 1]
        enum BoxPoly<A> {
            new {
                data: A
            }
        }
    }
}

impl_wrapper! {
    w::IoThen, w::Size, w::Any
}

impl_wrapper! {
    w::WhelkRequestLoopInitialState,
    w::WhelkRequestLoopContinuePredicate,
    w::WhelkRequestLoopStep
}
