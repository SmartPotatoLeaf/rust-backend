/// Generates `From` implementations from multiple source types to a SeaORM `ActiveModel` target.
///
/// This macro is specifically designed for SeaORM active models, automatically wrapping
/// all field values in `sea_orm::Set()` during conversion. It's the SeaORM-specific
/// variant of the `maps_to!` macro.
///
/// # Syntax
///
/// ```ignore
/// maps_set!(ActiveModelTarget {
///     field1, field2,
///     #into [field_that_needs_conversion]
/// }
/// #from [SourceStruct1, SourceStruct2, SourceStruct3]);
/// ```
///
/// # Fields
///
/// - **Direct fields**: Wrapped in `sea_orm::Set()` and copied/moved from source
/// - **#into []**: List of fields requiring conversion (`.into()`) before wrapping in `Set()`
/// - **#from []**: List of source structs from which `From` implementations will be generated
///
/// # Examples
///
/// ## Basic usage with SeaORM ActiveModel
///
/// ```ignore
/// # use spl_shared::maps_set;
/// # use sea_orm::ActiveValue;
/// #
/// // SeaORM ActiveModel (entity model)
/// struct UserActiveModel {
///     id: ActiveValue<i32>,
///     username: ActiveValue<String>,
///     email: ActiveValue<String>,
///     is_active: ActiveValue<bool>,
/// }
///
/// // DTO from API request
/// struct CreateUserDto {
///     id: i32,
///     username: String,
///     email: String,
///     is_active: bool,
/// }
///
/// // DTO from external service
/// struct ImportUserDto {
///     id: i32,
///     username: String,
///     email: String,
///     is_active: bool,
/// }
///
/// // Generates From<CreateUserDto> and From<ImportUserDto> for UserActiveModel
/// // Each field is automatically wrapped with sea_orm::Set()
/// maps_set!(UserActiveModel {
///     id, username, email, is_active
/// }
/// #from [CreateUserDto, ImportUserDto]);
///
/// // Usage:
/// let create_dto = CreateUserDto {
///     id: 1,
///     username: "alice".to_string(),
///     email: "alice@example.com".to_string(),
///     is_active: true,
/// };
/// let active_model: UserActiveModel = create_dto.into();
/// // Equivalent to:
/// // UserActiveModel {
/// //     id: sea_orm::Set(1),
/// //     username: sea_orm::Set("alice".to_string()),
/// //     email: sea_orm::Set("alice@example.com".to_string()),
/// //     is_active: sea_orm::Set(true),
/// // }
/// ```
///
/// ## Example with field conversions
///
/// ```ignore
/// # use spl_shared::maps_set;
/// # use sea_orm::ActiveValue;
/// # use uuid::Uuid;
/// #
/// // Custom type that needs conversion
/// #[derive(Clone)]
/// struct CompanyId(String);
///
/// impl From<i32> for CompanyId {
///     fn from(id: i32) -> Self { CompanyId(id.to_string()) }
/// }
///
/// impl From<uuid::Uuid> for CompanyId {
///     fn from(uuid: uuid::Uuid) -> Self { CompanyId(uuid.to_string()) }
/// }
///
/// // SeaORM ActiveModel
/// struct CompanyActiveModel {
///     id: ActiveValue<CompanyId>,
///     name: ActiveValue<String>,
///     revenue: ActiveValue<f64>,
/// }
///
/// // Source from legacy system (uses integer IDs)
/// struct LegacyCompanyDto {
///     id: i32,
///     name: String,
///     revenue: f64,
/// }
///
/// // Source from modern API (uses UUIDs)
/// struct ModernCompanyDto {
///     id: uuid::Uuid,
///     name: String,
///     revenue: f64,
/// }
///
/// // The 'id' field needs .into() conversion, then wrapped in Set()
/// // Other fields are directly wrapped in Set()
/// maps_set!(CompanyActiveModel {
///     name, revenue,
///     #into [id]
/// }
/// #from [LegacyCompanyDto, ModernCompanyDto]);
///
/// // Usage:
/// let legacy = LegacyCompanyDto {
///     id: 42,
///     name: "Acme Corp".to_string(),
///     revenue: 1_000_000.0,
/// };
/// let active_model: CompanyActiveModel = legacy.into();
/// // Equivalent to:
/// // CompanyActiveModel {
/// //     id: sea_orm::Set(CompanyId::from(42)),
/// //     name: sea_orm::Set("Acme Corp".to_string()),
/// //     revenue: sea_orm::Set(1_000_000.0),
/// // }
///
/// let modern = ModernCompanyDto {
///     id: uuid::Uuid::new_v4(),
///     name: "Tech Inc".to_string(),
///     revenue: 2_500_000.0,
/// };
/// let active_model2: CompanyActiveModel = modern.into();
/// ```
///
/// ## Real-world example: Multiple DTOs to ActiveModel
///
/// ```ignore
/// # use spl_shared::maps_set;
/// # use sea_orm::ActiveValue;
/// # use chrono::NaiveDateTime;
/// #
/// struct DiagnosticActiveModel {
///     plot_id: ActiveValue<uuid::Uuid>,
///     diagnosis_date: ActiveValue<NaiveDateTime>,
///     severity: ActiveValue<String>,
///     notes: ActiveValue<Option<String>>,
/// }
///
/// struct CreateDiagnosticDto {
///     plot_id: uuid::Uuid,
///     diagnosis_date: NaiveDateTime,
///     severity: String,
///     notes: Option<String>,
/// }
///
/// struct UpdateDiagnosticDto {
///     plot_id: uuid::Uuid,
///     diagnosis_date: NaiveDateTime,
///     severity: String,
///     notes: Option<String>,
/// }
///
/// struct ImportDiagnosticDto {
///     plot_id: uuid::Uuid,
///     diagnosis_date: NaiveDateTime,
///     severity: String,
///     notes: Option<String>,
/// }
///
/// // Convert all DTOs to the same ActiveModel
/// maps_set!(DiagnosticActiveModel {
///     plot_id, diagnosis_date, severity, notes
/// }
/// #from [CreateDiagnosticDto, UpdateDiagnosticDto, ImportDiagnosticDto]);
/// ```
#[macro_export]
macro_rules! maps_set {
    (
        $target:ident {
            $($field:ident),* $(,)?
        }
        #from [ $($source:tt)+ ]
    ) => {
        maps_set!(
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
                    $( $field: sea_orm::Set(s.$field), )*
                    $( $into_field: sea_orm::Set(s.$into_field.into()), )*
                }
            }
        }
        maps_set!(
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
                    $( $field: sea_orm::Set(s.$field), )*
                    $( $into_field: sea_orm::Set(s.$into_field.into()), )*
                }
            }
        }
    };
}