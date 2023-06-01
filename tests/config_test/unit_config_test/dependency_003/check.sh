#!/usr/bin/env bash

work_dir="$(dirname "$0")"
source "${work_dir}"/util_lib.sh

set +e

key_log_1='insert, key: "failure1.service", value: UnitReDep .*UnitOnFailureOf, "base.service"'
key_log_2='insert, key: "failure2.service", value: UnitReDep .*UnitOnFailureOf, "base.service"'
key_log_3='start the unit failure1.service'
key_log_4='start the unit failure2.service'

# usage: test OnFailure
function test01() {
    log_info "===== test01 ====="
    cp -arf "${work_dir}"/tmp_units/base.service ${SYSMST_LIB_PATH} || return 1
    cp -arf "${work_dir}"/tmp_units/base.service ${SYSMST_LIB_PATH}/failure1.service || return 1
    cp -arf "${work_dir}"/tmp_units/base.service ${SYSMST_LIB_PATH}/failure2.service || return 1
    sed -i '/Description/a OnFailure="failure1.service;failure2.service"' ${SYSMST_LIB_PATH}/base.service
    sed -i 's/sleep.*"/false"/' ${SYSMST_LIB_PATH}/base.service
    sctl daemon-reload
    echo > "${SYSMST_LOG}"
    sctl status failure1
    expect_eq $? 1
    sctl status failure2
    expect_eq $? 1
    sctl start base
    expect_eq $? 0 || return 1
    sleep 1
    sctl status base | grep Active | grep failed
    expect_eq $? 0
    sctl status base
    expect_eq $? 3
    sctl status failure1 failure2
    expect_eq $? 0
    check_log "${SYSMST_LOG}" "${key_log_1}" "${key_log_2}" "${key_log_3}" "${key_log_4}"
    expect_eq $? 0

    # clean
    sctl stop failure1 failure2
    check_status failure1 inactive
    expect_eq $? 0
    check_status failure2 inactive
    expect_eq $? 0
    sctl status failure1
    expect_eq $? 3
    sctl status failure2
    expect_eq $? 3

    # failed after reach start limit
    sed -i '/ExecStart/ a Restart="always"' ${SYSMST_LIB_PATH}/base.service
    sctl daemon-reload
    echo > "${SYSMST_LOG}"
    sctl restart base
    check_status base failed
    expect_eq $? 0 || return 1
    expect_eq "$(grep -a 'base.service, original state: Failed, change to: AutoRestart' "${SYSMST_LOG}" | wc -l)" 5 || cat "${SYSMST_LOG}"
    check_status failure1 active
    expect_eq $? 0
    check_status failure2 active
    expect_eq $? 0

    # unit not exist
    sed -i '/Restart/d' ${SYSMST_LIB_PATH}/base.service
    rm -rf ${SYSMST_LIB_PATH}/failure2.service
    sctl daemon-reload
    echo > "${SYSMST_LOG}"
    sctl restart base
    expect_eq $? 0 || return 1
    check_status base failed
    expect_eq $? 0 || return 1
    sctl status failure2
    expect_eq $? 1
    check_status failure1 active
    expect_eq $? 0 || return 1
    check_log "${SYSMST_LOG}" "${key_log_1}" "${key_log_2}" "${key_log_3}"
    expect_eq $? 0
    grep -a "${key_log_4}" "${SYSMST_LOG}"
    expect_eq $? 1

    # clean
    sctl stop failure1
}

# usage: test OnFailureJobMode
function test02() {
    log_info "===== test02 ====="

    # default: OnFailureJobMode=replace
    cp -arf "${work_dir}"/tmp_units/base.service ${SYSMST_LIB_PATH} || return 1
    cp -arf "${work_dir}"/tmp_units/base.service ${SYSMST_LIB_PATH}/failure1.service || return 1
    cp -arf "${work_dir}"/tmp_units/base.service ${SYSMST_LIB_PATH}/failure2.service || return 1
    cp -arf "${work_dir}"/tmp_units/base.service ${SYSMST_LIB_PATH}/failure3.service || return 1
    sed -i '/Description/a Conflicts="failure3.service"' ${SYSMST_LIB_PATH}/failure1.service
    sed -i '/Description/a OnFailure="failure1.service;failure2.service;failure3.service"' ${SYSMST_LIB_PATH}/base.service
    sed -i 's/sleep.*"/false"/' ${SYSMST_LIB_PATH}/base.service
    sctl daemon-reload
    echo > "${SYSMST_LOG}"
    sctl restart base
    check_status base failed
    expect_eq $? 0 || return 1
    check_status failure2 active
    expect_eq $? 0 || return 1
    line_1="$(grep -na 'start the unit failure1.service' ${SYSMST_LOG} | awk -F ':' '{print $1}')"
    line_3="$(grep -na 'start the unit failure3.service' ${SYSMST_LOG} | awk -F ':' '{print $1}')"
    if [ "${line_3}" -gt "${line_1}" ]; then
        check_status failure3 active
        expect_eq $? 0 || return 1
        check_status failure1 inactive
        expect_eq $? 0 || return 1
        check_log ${SYSMST_LOG} 'asdasdasd'
        expect_eq $? 0
    else
        check_status failure1 active
        expect_eq $? 0 || return 1
        check_status failure3 inactive
        expect_eq $? 0 || return 1
        check_log ${SYSMST_LOG} 'asdasdasd'
        expect_eq $? 0
    fi

    # clean
    sctl stop failure1 failure2 failure3
}

run_sysmaster || exit 1
test01 || exit 1
test02 || exit 1
exit "${EXPECT_FAIL}"
