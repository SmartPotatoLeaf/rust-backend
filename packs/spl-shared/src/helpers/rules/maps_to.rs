/// Generates unidirectional `From` implementations from multiple source types to a target type.
///
/// This macro allows creating conversions from several different types to a single target type,
/// useful when you have multiple representations that converge into the same model.
///
/// # Syntax
///
/// ```ignore
/// maps_to!(TargetStruct {
///     field1, field2,
///     #into [field_that_needs_conversion]
/// }
/// #from [SourceStruct1, SourceStruct2, SourceStruct3]);
/// ```
///
/// # Fields
///
/// - **Direct fields**: Copied/moved directly from the source
/// - **#into []**: List of fields that require conversion using `.into()`
/// - **#from []**: List of source structs from which `From` implementations will be generated
///
/// # Examples
///
/// ```ignore
/// # use spl_shared::maps_to;
/// #
/// // Unified target type
/// struct User {
///     id: i32,
///     name: String,
///     active: bool,
/// }
///
/// // Multiple sources with the same structure
/// struct DbUser {
///     id: i32,
///     name: String,
///     active: bool,
/// }
///
/// struct ApiUser {
///     id: i32,
///     name: String,
///     active: bool,
/// }
///
/// struct CachedUser {
///     id: i32,
///     name: String,
///     active: bool,
/// }
///
/// // Generates From<DbUser>, From<ApiUser>, and From<CachedUser> for User
/// maps_to!(User {
///     id, name, active
/// }
/// #from [DbUser, ApiUser, CachedUser]);
///
/// // Usage:
/// let db_user = DbUser { id: 1, name: "Bob".to_string(), active: true };
/// let user: User = db_user.into();
///
/// let api_user = ApiUser { id: 2, name: "Carol".to_string(), active: false };
/// let user2: User = api_user.into();
/// ```
///
/// ## Example with conversions
///
/// ```ignore
/// # use spl_shared::maps_to;
/// #
/// #[derive(Clone)]
/// struct ProductId(String);
///
/// impl From<i32> for ProductId {
///     fn from(id: i32) -> Self { ProductId(id.to_string()) }
/// }
///
/// impl From<String> for ProductId {
///     fn from(id: String) -> Self { ProductId(id) }
/// }
///
/// struct Product {
///     id: ProductId,
///     name: String,
///     price: f64,
/// }
///
/// struct LegacyProduct {
///     id: i32,
///     name: String,
///     price: f64,
/// }
///
/// struct ModernProduct {
///     id: String,
///     name: String,
///     price: f64,
/// }
///
/// // Converts both formats to Product, transforming id with .into()
/// maps_to!(Product {
///     name, price,
///     #into [id]
/// }
/// #from [LegacyProduct, ModernProduct]);
///
/// let legacy = LegacyProduct { id: 123, name: "Widget".to_string(), price: 9.99 };
/// let product: Product = legacy.into();
///
/// let modern = ModernProduct { id: "456".to_string(), name: "Gadget".to_string(), price: 19.99 };
/// let product2: Product = modern.into();
/// ```
#[macro_export]
macro_rules! maps_to {
    (
        $target:ident {
            $($field:ident),* $(,)?
        }
        #from [ $($source:tt)+ ]
    ) => {
        maps_to!(
            $target {
                $($field),*,
                #into []
            }
            #from [ $($source)+ ]
        );
    };


    (
        $target:ident {
            $($field:ident),* $(,)?
            #into [ $($into_field:ident),* $(,)? ]
        }
        #from [ $head:ident, $($tail:ident),+ ] // Head + Tail
    ) => {
        impl From<$head> for $target {
            fn from(s: $head) -> Self {
                Self {
                    $( $field: s.$field, )*
                    $( $into_field: s.$into_field.into(), )*
                }
            }
        }
        maps_to!(
            $target {
                $($field),*,
                #into [ $($into_field),* ]
            }
            #from [ $($tail),+ ]
        );
    };

    (
        $target:ident {
            $($field:ident),* $(,)?
            #into [ $($into_field:ident),* $(,)? ]
        }
        #from [ $head:ident ] // Solo Head
    ) => {
        impl From<$head> for $target {
            fn from(s: $head) -> Self {
                Self {
                    $( $field: s.$field, )*
                    $( $into_field: s.$into_field.into(), )*
                }
            }
        }
    };
}