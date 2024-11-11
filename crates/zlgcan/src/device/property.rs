#[doc = "  \\struct Options\n 节点mata的可选项。"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Options {
    #[doc = " 可选项的数据类型"]
    pub type_: *const ::core::ffi::c_char,
    #[doc = " 可选项的值"]
    pub value: *const ::core::ffi::c_char,
    #[doc = " 可选项的描述信息"]
    pub desc: *const ::core::ffi::c_char,
}

#[doc = "  \\struct Meta\n 节点mata信息。"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Meta {
    #[doc = " 配置项的数据类型"]
    pub type_: *const ::core::ffi::c_char,
    #[doc = " 配置项的说明性信息"]
    pub desc: *const ::core::ffi::c_char,
    #[doc = " 配置项是否是只读的，缺省为可读写"]
    pub read_only: ::core::ffi::c_int,
    #[doc = " 配置项输入格式的提示"]
    pub format: *const ::core::ffi::c_char,
    #[doc = " 对于数值类型的配置项来说是最小值，对字符串的配置项来说是最小长度（字节数）。"]
    pub min_value: f64,
    #[doc = " 对于数值类型的配置项来说是最大值，对字符串的配置项来说是最大长度（字节数）。"]
    pub max_value: f64,
    #[doc = " 配置项的单位"]
    pub unit: *const ::core::ffi::c_char,
    #[doc = " 通过旋钮/滚轮等方式修改配置项时的增量"]
    pub delta: f64,
    #[doc = " 配置项是否可见, true可见，false不可见，也可以绑定表达式（表达式使用参考demo3），缺省可见"]
    pub visible: *const ::core::ffi::c_char,
    #[doc = " 该配置项是否使能, true使能，false不使能，也可以绑定表达式（表达式使用参考demo3）。缺省使能"]
    pub enable: *const ::core::ffi::c_char,
    #[doc = " 配置项的可选值，仅但『type』为间接类型时有效"]
    pub editable: ::core::ffi::c_int,
    #[doc = " 配置项的可选值，仅但『type』为间接类型时有效，以NULL结束"]
    pub options: *mut *mut Options,
}

#[doc = "  \\struct Pair\n  属性的KeyValue对。"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Pair {
    pub key: *const ::core::ffi::c_char,
    pub value: *const ::core::ffi::c_char,
}

#[doc = "  \\struct ConfigNode\n  ConfigNode"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ConfigNode {
    #[doc = " 节点的名字"]
    pub name: *const ::core::ffi::c_char,
    #[doc = " 节点的值 同样可以绑定表达式"]
    pub value: *const ::core::ffi::c_char,
    #[doc = " 节点值的表达式，当有该表达式时，value由此表达式计算而来"]
    pub binding_value: *const ::core::ffi::c_char,
    #[doc = " 该节点的路径"]
    pub path: *const ::core::ffi::c_char,
    #[doc = " 配置项信息"]
    pub meta_info: *mut Meta,
    #[doc = " 该节点的子节点, 以NULL结束"]
    pub children: *mut *mut ConfigNode,
    #[doc = " 该节点的属性, 以NULL结束"]
    pub attributes: *mut *mut Pair,
}

#[doc = " \\brief 设置指定路径的属性的值。\n \\param[in] path  : 属性的路径。\n \\param[in] value : 属性的值。\n\n \\retval 成功返回1，失败返回0。"]
pub type SetValueFunc = ::core::option::Option<
    unsafe extern "C" fn(
        path: *const ::core::ffi::c_char,
        value: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int,
>;

#[doc = " \\brief 获取指定路径的属性的值。\n \\param[in] path  : 属性的路径。\n \\retval 成功返回属性的值，失败返回NULL。"]
pub type GetValueFunc = ::core::option::Option<
    unsafe extern "C" fn(path: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char,
>;

#[doc = " \\brief 获取属性的描述信息。\n\n \\retval ConfigNode"]
pub type GetPropertiesFunc = ::core::option::Option<unsafe extern "C" fn() -> *const ConfigNode>;

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct IProperty {
    pub SetValue: SetValueFunc,
    pub GetValue: GetValueFunc,
    pub GetProperties: GetPropertiesFunc,
}



