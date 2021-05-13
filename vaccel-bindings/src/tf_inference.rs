use crate::{
    vaccel_session, vaccel_tf_buffer, vaccel_tf_model, vaccel_tf_model_load_graph,
    vaccel_tf_model_run, vaccel_tf_node, vaccel_tf_status, vaccel_tf_tensor, VACCEL_OK,
};
use crate::{Error, Result};

impl vaccel_tf_model {
    /// Load a TensorFlow graph from a model
    ///
    /// The TensorFlow model must have been created and registered to
    /// a session. The operation will load the graph and keep the graph
    /// TensorFlow representation in the model struct
    ///
    /// # Arguments
    ///
    /// * `session` - The session in the context of which we perform the operation. The model needs
    /// to be registered with this session.
    ///
    /// # Examples
    ///
    /// ```
    /// use vaccel_bindings::{vaccel_session, vaccel_tf_model, Error};
    ///
    /// fn main() -> Result<(), vaccel_bindings::Error> {
    ///     let path = Path::new("/path/to/model.pb");
    ///     let model = vaccel_tf_model::new(path)?;
    ///
    ///     let sess = vaccel_session::new(0)?;
    ///     sess.register(model)?;
    ///
    ///     model.load_graph(sess)?;
    ///         
    ///     println!("Successfully load TensorFlow graph");
    ///     Ok(())
    /// }
    ///
    /// ```
    ///
    pub fn load_graph(&mut self, sess: &mut vaccel_session) -> Result<vaccel_tf_status> {
        let mut status = vaccel_tf_status::default();

        match unsafe { vaccel_tf_model_load_graph(sess, self, &mut status) as u32 } {
            VACCEL_OK => Ok(status),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Run inference on a TensorFlow model
    ///
    /// This will run inference using a TensorFlow graph that has been previously loaded
    /// using `vaccel_tf_model::load_graph`.
    ///
    pub fn inference(
        &self,
        sess: &mut vaccel_session,
        run_options: &vaccel_tf_buffer,
        in_nodes: &[vaccel_tf_node],
        in_tensors: &[vaccel_tf_tensor],
        out_nodes: &[vaccel_tf_node],
    ) -> Result<(Vec<vaccel_tf_tensor>, vaccel_tf_status)> {
        let mut status = vaccel_tf_status::default();
        let mut out_tensors: Vec<vaccel_tf_tensor> = vec![Default::default(); out_nodes.len()];

        match unsafe {
            vaccel_tf_model_run(
                sess,
                self,
                run_options,
                in_nodes.as_ptr(),
                in_tensors.as_ptr(),
                in_nodes.len() as i32,
                out_nodes.as_ptr(),
                out_tensors.as_mut_ptr(),
                out_nodes.len() as i32,
                &mut status,
            ) as u32
        } {
            VACCEL_OK => Ok((out_tensors, status)),
            err => Err(Error::Runtime(err)),
        }
    }
}
