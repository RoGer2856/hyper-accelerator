use std::sync::Arc;

use crate::application_context_trait::ApplicationContextTrait;

pub trait RequestContextTrait<ApplicationContextType: ApplicationContextTrait>:
    Send + 'static
{
    fn create(app_context: Arc<ApplicationContextType>) -> Self;
}
