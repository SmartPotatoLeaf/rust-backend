/// Generates bidirectional `From` implementations between two structs.
///
/// This macro creates automatic conversions in both directions between two types,
/// allowing fields to be mapped directly or using `.into()` when needed.
///
/// # Syntax
///
/// ```ignore
/// map_mirror!(Struct1, Struct2 {
///     field1, field2, field3,
///     #into [field_that_needs_conversion]
/// });
/// ```
///
/// # Fields
///
/// - **Direct fields**: Copied/moved directly without transformation
/// - **#into []**: List of fields that require conversion using `.into()`
///
/// # Examples
///
/// ```ignore
/// # use spl_shared::map_mirror;
/// #
/// // Domain and DTO structures
/// #[derive(Debug, Clone)]
/// struct UserId(i32);
///
/// impl From<i32> for UserId {
///     fn from(id: i32) -> Self { UserId(id) }
/// }
///
/// impl From<UserId> for i32 {
///     fn from(user_id: UserId) -> Self { user_id.0 }
/// }
///
/// struct DomainUser {
///     id: UserId,
///     name: String,
///     email: String,
/// }
///
/// struct DtoUser {
///     id: i32,
///     name: String,
///     email: String,
/// }
///
/// // Generates From<DomainUser> for DtoUser and From<DtoUser> for DomainUser
/// map_mirror!(DomainUser, DtoUser {
///     name, email,
///     #into [id]
/// });
///
/// // Usage:
/// let domain = DomainUser {
///     id: UserId(1),
///     name: "Alice".to_string(),
///     email: "alice@example.com".to_string(),
/// };
///
/// let dto: DtoUser = domain.into();
/// let back_to_domain: DomainUser = dto.into();
/// ```
///
/// ## Example without conversions (direct fields only)
///
/// ```ignore
/// # use spl_shared::map_mirror;
/// #
/// struct Point2D {
///     x: f64,
///     y: f64,
/// }
///
/// struct Coordinate {
///     x: f64,
///     y: f64,
/// }
///
/// // All fields are directly compatible
/// map_mirror!(Point2D, Coordinate {
///     x, y
/// });
///
/// let point = Point2D { x: 10.0, y: 20.0 };
/// let coord: Coordinate = point.into();
/// ```
#[macro_export]
macro_rules! map_mirror {
    ($struct1:ident, $struct2:ident {
        $( $direct_field:ident ),* $(,)?
        #into [ $( $into_field:ident ),* $(,)? ]
    }) => {
        impl From<$struct1> for $struct2 {
            fn from(s: $struct1) -> Self {
                Self {
                    $( $direct_field: s.$direct_field, )*
                    $( $into_field: s.$into_field.into(), )*
                }
            }
        }
        impl From<$struct2> for $struct1 {
            fn from(s: $struct2) -> Self {
                Self {
                    $( $direct_field: s.$direct_field, )*
                    $( $into_field: s.$into_field.into(), )*
                }
            }
        }
    };

    ($struct1:ident, $struct2:ident {
        $( $direct_field:ident ),* $(,)?
    }) => {
        map_mirror!(
            $struct1, $struct2 {
                $( $direct_field ),*,
                #into []
            }
        );
    };
}
