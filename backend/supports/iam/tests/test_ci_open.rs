use bios_iam::basic::dto::iam_open_dto::{IamOpenAddOrModifyProductReq, IamOpenAddSpecReq, IamOpenAkSkAddReq, IamOpenBindAkProductReq};
use bios_iam::basic::serv::iam_open_serv::IamOpenServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::log::info;

use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext) -> TardisResult<()> {
    let funs = iam_constants::get_tardis_inst();

    let product_code = "product1".to_string();
    let spec1_code = "spec1".to_string();
    let spec2_code = "spec2".to_string();
    info!("【test_ci_open】 : Add Product");
    IamOpenServ::add_or_modify_product(
        &IamOpenAddOrModifyProductReq {
            code: TrimString(product_code.clone()),
            name: TrimString("测试产品".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            specifications: vec![
                IamOpenAddSpecReq {
                    code: TrimString(spec1_code.clone()),
                    name: TrimString("测试规格1".to_string()),
                    icon: None,
                    url: None,
                    scope_level: None,
                    disabled: None,
                },
                IamOpenAddSpecReq {
                    code: TrimString(spec2_code.clone()),
                    name: TrimString("测试规格2".to_string()),
                    icon: None,
                    url: None,
                    scope_level: None,
                    disabled: None,
                },
            ],
        },
        &funs,
        context1,
    )
    .await?;
    IamOpenServ::add_or_modify_product(
        &IamOpenAddOrModifyProductReq {
            code: TrimString(product_code.clone()),
            name: TrimString("测试产品".to_string()),
            icon: None,
            scope_level: None,
            disabled: None,
            specifications: vec![
                IamOpenAddSpecReq {
                    code: TrimString(spec1_code.clone()),
                    name: TrimString("测试规格1".to_string()),
                    icon: None,
                    url: None,
                    scope_level: None,
                    disabled: None,
                },
                IamOpenAddSpecReq {
                    code: TrimString(spec2_code.clone()),
                    name: TrimString("测试规格2".to_string()),
                    icon: None,
                    url: None,
                    scope_level: None,
                    disabled: None,
                },
            ],
        },
        &funs,
        context1,
    )
    .await?;
    info!("【test_ci_open】 : Apply ak/sk");
    let cert_resp = IamOpenServ::general_cert(
        IamOpenAkSkAddReq {
            tenant_id: context1.own_paths.clone(),
            app_id: None,
        },
        &funs,
        context1,
    )
    .await?;
    let cert_id = cert_resp.id;
    let _cert_resp2 = IamOpenServ::general_cert(
        IamOpenAkSkAddReq {
            tenant_id: context1.own_paths.clone(),
            app_id: None,
        },
        &funs,
        context1,
    )
    .await?;
    IamOpenServ::bind_cert_product_and_spec(
        &cert_id,
        &IamOpenBindAkProductReq {
            product_code: product_code.clone(),
            spec_code: spec1_code.clone(),
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            api_call_frequency: Some(500),
            api_call_count: Some(10000),
        },
        &funs,
        context1,
    )
    .await?;
    info!("【test_ci_open】 : Get account info");
    let global_ctx = TardisContext {
        own_paths: "".to_string(),
        ..Default::default()
    };
    let info = IamOpenServ::get_rule_info(Some(cert_id.clone()), None, &funs, &global_ctx).await?;
    assert_eq!(info.cert_id, cert_id.clone());
    assert_eq!(info.spec_code, spec1_code.clone());
    assert_eq!(info.api_call_frequency, Some(500));
    assert_eq!(info.api_call_count, Some(10000));
    Ok(())
}
