use proc_macro::*;
use quote::ToTokens;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, TypePath};

#[proc_macro_derive(Serialize, attributes(Belonging))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let struct_name = get_struct_name(&input);
    let fields = get_fields(&input);
    let attribute = parse_attr(&input);

    let mut contents = String::new();
    fields.iter().for_each(|f| {
        let field_name = f.ident.clone().unwrap().to_string();

        let length = format!(
            "content_buffer.write_u32(u32::try_from(self.{field_name}.as_bytes().len())?).await?;"
        );
        let extend = format!("content_buffer.extend(self.{field_name}.as_bytes());");
        contents.push_str(&format!("{length}\n{extend}\n"));
    });

    format!(
        "#[async_trait]
        impl Serialize for {struct_name} {{
            async fn serialize(&self) -> Result<Vec<u8>, SerializerError> {{
                let mut buffer: Vec<u8> = Vec::new();
                
                // MessageType
                buffer.write_u8({attribute}::{struct_name} as u8).await?;

                let mut content_buffer: Vec<u8> = Vec::new();
                {contents}

                // Misc.
                buffer.write_u32(u32::try_from(content_buffer.len())?).await?;

                buffer.append(&mut content_buffer);
                Ok(buffer)
            }}
    }}",
    )
    .parse()
    .unwrap()
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let struct_name = get_struct_name(&input);
    let fields = get_fields(&input);
    let attribute = parse_attr(&input);

    let mut variables = String::new();
    fields.iter().for_each(|f| {
        let field_name = f.ident.clone().unwrap().to_string();
        variables.push_str(&format!(
            "let {field_name} = read_string_from_buffer(&mut inner_cursor).await?;\n"
        ));
    });

    let mut return_string = String::from("return Ok(Self {\n");
    fields.iter().for_each(|f| {
        let field_name = f.ident.clone().unwrap().to_string();
        return_string.push_str(&format!("{field_name}: {field_name}.unwrap(),\n"));
    });
    return_string.push_str("});");

    format!(
        "#[async_trait]
        impl Deserialize for {struct_name} {{
            async fn deserialize<'a>(data: &'a [u8]) -> Result<Self, DeserializerError>
            where
                Self: Sized
            {{
                if data.is_empty() {{
                    return Err(DeserializerError::InvalidBufferLength);
                }}
                let mut data = Cursor::new(data);

                let msg_type = data.read_u8().await?;
                let message_type = {attribute}::from(msg_type);
                if message_type != {attribute}::{struct_name} {{
                    return Err(DeserializerError::InvalidMessageType);
                }}

                let mut inner_cursor = prepare_inner_cursor(&mut data).await?;
                {variables}

                //TODO IMPORTANT!! ADD #.is_none() checks and return 
                //`return Err(DeserializerError::InvalidData);`

                {return_string}
            }}
        }}"
    )
    .parse()
    .unwrap()
}

fn get_fields(input: &DeriveInput) -> &Fields {
    match &input.data {
        Data::Struct(DataStruct {
            struct_token: _,
            fields,
            ..
        }) => fields,
        _ => {
            panic!("(De)Serialization only works for structs, but none was found!")
        }
    }
}

fn get_struct_name(input: &DeriveInput) -> String {
    match &input.data {
        Data::Struct(_) => input.ident.to_string(),
        _ => {
            panic!("(De)Serialization only works for structs, but none was found!")
        }
    }
}

fn parse_attr(input: &DeriveInput) -> String {
    if input.attrs.len() != 1 {
        panic!(
            "Struct must have exactly one attribute: either ClientMessageType or ServerMessageType"
        );
    }

    let attr = input.attrs.first().unwrap();
    if attr.path().is_ident("Belonging") {
        let args = attr.parse_args::<TypePath>().unwrap();
        let message_type = args.to_token_stream().to_string();

        if message_type != "ClientMessageType" && message_type != "ServerMessageType" {
            panic!("Belonging attribute argument must be either 'ClientMessageType' or 'ServerMessageType'");
        }

        message_type
    } else {
        panic!("Only the Belonging attribute is supported");
    }
}
