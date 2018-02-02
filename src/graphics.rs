// add image
// modify image color GREEN to that
// draw image 2D transformation + z

pub struct Graphics {
}

// TODO: put in resource
pub struct GraphicsResource {
}

impl GraphicsResource {
}

impl Graphics {
    pub fn init() -> (Graphics, GraphicsResource) {
    }

    pub fn acquire_next_frame(&mut self, drawer: &mut Drawer) {
        // TODO: cleanup finished before acquiring next image
    }

    pub fn render_frame(&mut self, drawer: &mut Drawer) {
    }
}

// IN PEPE:
// we have a rendering resource
// and a graphics resource
// rendering contains command buffer
// graphics contains assets and things
//
// if we only draw a predefined set of image then it should be easy
