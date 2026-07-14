use super::net::Uri;

jni::bind_java_type! {
    pub DocumentsContract => android.provider.DocumentsContract,
    type_map {
        Uri => android.net.Uri,
    },
    methods {
        static fn get_tree_document_id(uri: Uri) -> JString,
        static fn build_child_documents_uri_using_tree(tree_uri: Uri, document_id: JString) -> Uri,
        static fn build_document_uri_using_tree(tree_uri: Uri, document_id: JString) -> Uri,
    }
}

jni::bind_java_type! {
    pub DocumentsContractDocument => "android.provider.DocumentsContract$Document",
    fields {
        #[allow(non_snake_case)]
        static COLUMN_DOCUMENT_ID: JString,
        #[allow(non_snake_case)]
        static COLUMN_DISPLAY_NAME: JString,
        #[allow(non_snake_case)]
        static COLUMN_MIME_TYPE: JString,
    }
}
