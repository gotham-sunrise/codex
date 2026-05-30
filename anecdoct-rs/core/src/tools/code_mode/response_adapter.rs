use anecdoct_code_mode::ImageDetail as CodeModeImageDetail;
use anecdoct_protocol::models::DEFAULT_IMAGE_DETAIL;
use anecdoct_protocol::models::FunctionCallOutputContentItem;
use anecdoct_protocol::models::ImageDetail;

trait IntoProtocol<T> {
    fn into_protocol(self) -> T;
}

pub(super) fn into_function_call_output_content_items(
    items: Vec<anecdoct_code_mode::FunctionCallOutputContentItem>,
) -> Vec<FunctionCallOutputContentItem> {
    items.into_iter().map(IntoProtocol::into_protocol).collect()
}

impl IntoProtocol<ImageDetail> for CodeModeImageDetail {
    fn into_protocol(self) -> ImageDetail {
        let value = self;
        match value {
            CodeModeImageDetail::High => ImageDetail::High,
            CodeModeImageDetail::Original => ImageDetail::Original,
        }
    }
}

impl IntoProtocol<FunctionCallOutputContentItem>
    for anecdoct_code_mode::FunctionCallOutputContentItem
{
    fn into_protocol(self) -> FunctionCallOutputContentItem {
        let value = self;
        match value {
            anecdoct_code_mode::FunctionCallOutputContentItem::InputText { text } => {
                FunctionCallOutputContentItem::InputText { text }
            }
            anecdoct_code_mode::FunctionCallOutputContentItem::InputImage { image_url, detail } => {
                FunctionCallOutputContentItem::InputImage {
                    image_url,
                    detail: detail
                        .map(IntoProtocol::into_protocol)
                        .or(Some(DEFAULT_IMAGE_DETAIL)),
                }
            }
        }
    }
}
