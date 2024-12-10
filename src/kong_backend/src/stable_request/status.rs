use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub status_code: StatusCode,
    pub message: Option<String>,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{} - {}", self.status_code, message),
            None => write!(f, "{}", self.status_code),
        }
    }
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize)]
pub enum StatusCode {
    Start,
    // add pool
    AddToken0,
    AddToken0Success,
    AddToken0Failed,
    AddLPToken,
    AddLPTokenSuccess,
    AddLPTokenFailed,
    AddPool,
    AddPoolSuccess,
    AddPoolFailed,
    // add liquidity
    SendToken0,
    SendToken0Success,
    SendToken0Failed,
    VerifyToken0,
    VerifyToken0Success,
    VerifyToken0Failed,
    Token0NotFound,
    ReturnUnusedToken0,
    ReturnUnusedToken0Success,
    ReturnUnusedToken0Failed,
    ReturnToken0,
    ReturnToken0Success,
    ReturnToken0Failed,
    SendToken1,
    SendToken1Success,
    SendToken1Failed,
    VerifyToken1,
    VerifyToken1Success,
    VerifyToken1Failed,
    Token1NotFound,
    PoolNotFound,
    ReturnUnusedToken1,
    ReturnUnusedToken1Success,
    ReturnUnusedToken1Failed,
    ReturnToken1,
    ReturnToken1Success,
    ReturnToken1Failed,
    // remove liquidity
    RemoveLiquidityFromPool,
    ReturnUserLPTokenAmount,
    ReturnUserLPTokenAmountSuccess,
    ReturnUserLPTokenAmountFailed,
    ReceiveToken0,
    ReceiveToken0Success,
    ReceiveToken0Failed,
    ReceiveToken1,
    ReceiveToken1Success,
    ReceiveToken1Failed,
    // swap
    PayTokenNotFound,
    PayTxIdNotSupported,
    PayTxIdNotFound,
    PayTokenAmountIsZero,
    ReceiveTokenNotFound,
    ReceiveAddressNotFound,
    SendPayToken,
    SendPayTokenSuccess,
    SendPayTokenFailed,
    VerifyPayToken,
    VerifyPayTokenSuccess,
    VerifyPayTokenFailed,
    SendReceiveToken,
    SendReceiveTokenSuccess,
    SendReceiveTokenFailed,
    ReturnPayToken,
    ReturnPayTokenSuccess,
    ReturnPayTokenFailed,
    // claim
    ClaimToken,
    ClaimTokenSuccess,
    ClaimTokenFailed,
    // pool amounts
    CalculatePoolAmounts,
    CalculatePoolAmountsSuccess,
    CalculatePoolAmountsFailed,
    UpdatePoolAmounts,
    UpdatePoolAmountsSuccess,
    UpdatePoolAmountsFailed,
    // user LP token amount
    UpdateUserLPTokenAmount,
    UpdateUserLPTokenAmountSuccess,
    UpdateUserLPTokenAmountFailed,
    // send LP token
    SendLPTokenToUser,
    SendLPTokenToUserSuccess,
    SendLPTokenToUserFailed,
    // general
    Success,
    Failed,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            StatusCode::Start => write!(f, "Started"),
            StatusCode::AddToken0 => write!(f, "Adding token 0"),
            StatusCode::AddToken0Success => write!(f, "Token 0 added"),
            StatusCode::AddToken0Failed => write!(f, "Failed adding token 0"),
            StatusCode::AddLPToken => write!(f, "Adding LP token"),
            StatusCode::AddLPTokenSuccess => write!(f, "LP token added"),
            StatusCode::AddLPTokenFailed => write!(f, "Failed adding LP token"),
            StatusCode::AddPool => write!(f, "Adding pool"),
            StatusCode::AddPoolSuccess => write!(f, "Pool added"),
            StatusCode::AddPoolFailed => write!(f, "Failed adding pool"),
            StatusCode::SendToken0 => write!(f, "Sending token 0"),
            StatusCode::SendToken0Success => write!(f, "Token 0 sent"),
            StatusCode::SendToken0Failed => write!(f, "Failed sending token 0"),
            StatusCode::VerifyToken0 => write!(f, "Verifying token 0"),
            StatusCode::VerifyToken0Success => write!(f, "Token 0 verified"),
            StatusCode::VerifyToken0Failed => write!(f, "Failed verifying token 0"),
            StatusCode::Token0NotFound => write!(f, "Invalid token 0"),
            StatusCode::ReturnUnusedToken0 => write!(f, "Returning unused token 0"),
            StatusCode::ReturnUnusedToken0Success => write!(f, "Unused token 0 returned"),
            StatusCode::ReturnUnusedToken0Failed => write!(f, "Failed sending unused token 0"),
            StatusCode::ReturnToken0 => write!(f, "Returning token 0"),
            StatusCode::ReturnToken0Success => write!(f, "Token 0 returned"),
            StatusCode::ReturnToken0Failed => write!(f, "Failed returning token 0"),
            StatusCode::SendToken1 => write!(f, "Sending token 1"),
            StatusCode::SendToken1Success => write!(f, "Token 1 sent"),
            StatusCode::SendToken1Failed => write!(f, "Failed sending token 1"),
            StatusCode::VerifyToken1 => write!(f, "Verifying token 1"),
            StatusCode::VerifyToken1Success => write!(f, "Token 1 verified"),
            StatusCode::VerifyToken1Failed => write!(f, "Failed verifying token 1"),
            StatusCode::Token1NotFound => write!(f, "Invalid token 1"),
            StatusCode::PoolNotFound => write!(f, "Pool not found"),
            StatusCode::ReturnUnusedToken1 => write!(f, "Returning unused token 1"),
            StatusCode::ReturnUnusedToken1Success => write!(f, "Unused token 1 returned"),
            StatusCode::ReturnUnusedToken1Failed => write!(f, "Failed sending unused token 1"),
            StatusCode::ReturnToken1 => write!(f, "Returning token 1"),
            StatusCode::ReturnToken1Success => write!(f, "Token 1 returned"),
            StatusCode::ReturnToken1Failed => write!(f, "Failed sending token 1"),
            StatusCode::RemoveLiquidityFromPool => write!(f, "Remove liquidity from pool"),
            StatusCode::ReturnUserLPTokenAmount => write!(f, "Returning user LP token amount"),
            StatusCode::ReturnUserLPTokenAmountSuccess => write!(f, "User LP token amount returned"),
            StatusCode::ReturnUserLPTokenAmountFailed => write!(f, "Failed returning user LP token amount"),
            StatusCode::ReceiveToken0 => write!(f, "Receiving token 0"),
            StatusCode::ReceiveToken0Success => write!(f, "Token 0 received"),
            StatusCode::ReceiveToken0Failed => write!(f, "Failed receiving token 0"),
            StatusCode::ReceiveToken1 => write!(f, "Receiving token 1"),
            StatusCode::ReceiveToken1Success => write!(f, "Token 1 received"),
            StatusCode::ReceiveToken1Failed => write!(f, "Failed receiving token 1"),
            StatusCode::PayTokenNotFound => write!(f, "Invalid pay token"),
            StatusCode::PayTxIdNotSupported => write!(f, "Pay tx id not supported"),
            StatusCode::PayTxIdNotFound => write!(f, "Pay tx id not found"),
            StatusCode::PayTokenAmountIsZero => write!(f, "Pay token amount is zero"),
            StatusCode::ReceiveTokenNotFound => write!(f, "Invalid receive token"),
            StatusCode::ReceiveAddressNotFound => write!(f, "Invalid receive address"),
            StatusCode::SendPayToken => write!(f, "Sending pay token"),
            StatusCode::SendPayTokenSuccess => write!(f, "Pay token sent"),
            StatusCode::SendPayTokenFailed => write!(f, "Failed sending pay token"),
            StatusCode::VerifyPayToken => write!(f, "Verifying pay token"),
            StatusCode::VerifyPayTokenSuccess => write!(f, "Pay token verified"),
            StatusCode::VerifyPayTokenFailed => write!(f, "Failed verifying pay token"),
            StatusCode::SendReceiveToken => write!(f, "Receiving receive token"),
            StatusCode::SendReceiveTokenSuccess => write!(f, "Receive token received"),
            StatusCode::SendReceiveTokenFailed => write!(f, "Failed receiving receive token"),
            StatusCode::ReturnPayToken => write!(f, "Returning pay token"),
            StatusCode::ReturnPayTokenSuccess => write!(f, "Pay token returned"),
            StatusCode::ReturnPayTokenFailed => write!(f, "Failing returning pay token"),
            StatusCode::ClaimToken => write!(f, "Claiming token"),
            StatusCode::ClaimTokenSuccess => write!(f, "Token claimed"),
            StatusCode::ClaimTokenFailed => write!(f, "Failed claiming token"),
            StatusCode::CalculatePoolAmounts => write!(f, "Calculating pool amounts"),
            StatusCode::CalculatePoolAmountsSuccess => write!(f, "Pool amounts calculated"),
            StatusCode::CalculatePoolAmountsFailed => write!(f, "Failed calculating pool amounts"),
            StatusCode::UpdatePoolAmounts => write!(f, "Updating liquidity pool"),
            StatusCode::UpdatePoolAmountsSuccess => write!(f, "Liquidity pool updated"),
            StatusCode::UpdatePoolAmountsFailed => write!(f, "Failed updating liquidity pool"),
            StatusCode::UpdateUserLPTokenAmount => write!(f, "Updating user LP token amount"),
            StatusCode::UpdateUserLPTokenAmountSuccess => write!(f, "User LP token amount updated"),
            StatusCode::UpdateUserLPTokenAmountFailed => write!(f, "Failed updating user LP token amount"),
            StatusCode::SendLPTokenToUser => write!(f, "Sending LP token to user"),
            StatusCode::SendLPTokenToUserSuccess => write!(f, "LP token sent to user"),
            StatusCode::SendLPTokenToUserFailed => write!(f, "Failed sending LP token to user"),
            StatusCode::Success => write!(f, "Success"),
            StatusCode::Failed => write!(f, "Failed"),
        }
    }
}
